use super::*;
use futures::{future, Stream};
use hyper::rt::Future;
use hyper::{Body, Error, Method, Request, Response, StatusCode};
use std::collections::HashMap;
use std::fs::read_dir;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

type ResponseFuture = Box<Future<Item = Response<Body>, Error = io::Error> + Send>;

pub fn list_files(path: PathBuf) -> Result<Vec<PathBuf>, io::Error> {
    let result = if path.is_dir() {
        read_dir(path)?
            .flatten()
            .map(|x| x.path())
            .flat_map(list_files)
            .flatten()
            .collect()
    } else {
        vec![path]
    };
    Ok(result)
}

#[derive(Debug, Clone)]
pub struct Router {
    games: Arc<Mutex<HashMap<String, Game>>>,
    static_dir: PathBuf,
    access_key: Arc<Mutex<Option<AccessKey>>>,
}

#[derive(Debug, Deserialize, Clone)]
struct StartGameRequest {
    id: String,
}
#[derive(Debug, Deserialize, Clone)]
struct PasswordRequest {
    password: String,
}

#[derive(Debug, Clone)]
struct AccessKey {
    access_key: String,
    access_time: u64,
}

impl AccessKey {
    pub fn new(access_key: String, access_time: u64) -> Self {
        AccessKey {
            access_key: access_key.clone(),
            access_time: access_time,
        }
    }
}

/*
 * TODO: determine if this chunk of commented code is necessary
struct CheckPasswordOutput {
    success: bool,
    access_key: Option<String>,
}

impl CheckPasswordOutput {
    pub fn new(success: bool, access_key: Vec<u8>) -> Self {
        CheckPasswordOutput {
            success: success,
            access_key: access_key,
        }
    }
}
*/

impl hyper::service::Service for Router {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = io::Error;
    type Future = ResponseFuture;
    fn call(&mut self, req: Request<Self::ReqBody>) -> Self::Future {
        self.route(req)
    }
}

impl hyper::service::NewService for Router {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = io::Error;
    type Service = Router;
    type Future = Box<Future<Item = Self::Service, Error = Self::InitError> + Send>;
    type InitError = Error;
    fn new_service(&self) -> Self::Future {
        Box::new(future::ok(Self {
            games: self.games.clone(),
            static_dir: self.static_dir.clone(),
            access_key: self.access_key.clone(),
        }))
    }
}

impl Router {
    pub fn new(config: Config) -> Self {
        Router {
            games: Arc::new(Mutex::new(config.games)),
            static_dir: config.static_dir,
            access_key: Arc::new(Mutex::new(None)),
        }
    }

    fn invalid_endpoint(&self) -> ResponseFuture {
        Box::new(future::result(
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("uwu 404 not foundu hiss".to_owned()))
                .map_err(|err| {
                    io::Error::new(
                        ErrorKind::Other,
                        format!("An error occured when constructing 404 error: {}", err),
                    )
                }),
        ))
    }

    fn list_games(&self) -> ResponseFuture {
        let (body, status) = match self
            .games
            .lock()
            .map_err(|err| {
                io::Error::new(
                    ErrorKind::Other,
                    format!("Failed to acquire mutex on games list: {}", err),
                )
            }).and_then(|games| {
                serde_json::to_string(&*games).map_err(|err| io::Error::new(ErrorKind::Other, err))
            }).map(Body::from)
        {
            Ok(v) => (v, StatusCode::OK),
            Err(_e) => (
                Body::from("Internal server error".to_owned()),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
        };
        Box::new(future::result(
            Response::builder()
                .status(status)
                .header(hyper::header::CONTENT_TYPE, "application/json")
                .body(body)
                .map_err(|_e| {
                    io::Error::new(
                        ErrorKind::Other,
                        "Failed to acquire mutex lock on games list".to_owned(),
                    )
                }),
        ))
    }

    fn start_game(&self, request: Request<Body>) -> ResponseFuture {
        let games = self.games.clone();

        let response = request
            .into_body()
            .concat2()
            .map_err(|err| {
                io::Error::new(
                    ErrorKind::Other,
                    format!("Failed to parse byte string: {}", err),
                )
            }).and_then(|body| {
                serde_json::from_slice(&body).map_err(|err| io::Error::new(ErrorKind::Other, err))
            }).and_then(move |request_body: StartGameRequest| {
                let games = games.lock().map_err(|_e| {
                    io::Error::new(
                        ErrorKind::Other,
                        "Failed to acquire mutex lock on games list".to_owned(),
                    )
                });

                games.map(|games| {
                    let game = games.get(&request_body.id).ok_or_else(|| {
                        io::Error::new(
                            ErrorKind::Other,
                            "Failed to find game in list of available games".to_owned(),
                        )
                    })?;

                    let exe_path = game.exe_path.clone();
                    let exe_args = game.exe_args.clone();

                    Command::new(exe_path).args(exe_args).spawn()?;

                    Response::builder()
                        .status(StatusCode::OK)
                        .body(Body::from("Starting game!".to_owned()))
                        .map_err(|err| {
                            io::Error::new(
                                ErrorKind::Other,
                                format!("An error occured when building a response: {}", err),
                            )
                        })
                })
            }).flatten();
        Box::new(response)
    }

    // Checks password at demo screen
    // If correct, returns serialized access key in the ResponseFuture
    fn check_password(&mut self, request: Request<Body>, salted_hash: String) -> ResponseFuture {
        let access_key = self.access_key.clone();

        let response = request
            .into_body()
            .concat2()
            .map_err(|err| {
                io::Error::new(
                    ErrorKind::Other,
                    format!("Failed to parse byte string: {}", err),
                )
            }).and_then(|body| {
                serde_json::from_slice(&body).map_err(|err| io::Error::new(ErrorKind::Other, err))
            }).and_then(move |request_body: PasswordRequest| {
                let password = request_body.password.clone();

                let outgoing_json: String =
                    if check_password(password.to_string(), salted_hash.as_str().as_bytes()) {
                        // TODO: this will be some string of 64 bytes encoded as hex
                        let session_token: String = String::new();

                        let mut guard = access_key.lock().map_err(|err| {
                            io::Error::new(
                                ErrorKind::Other,
                                format!("Failed to acquire mutex on games list: {}", err),
                            )
                        })?;

                        *guard = Some(AccessKey::new(
                            session_token.clone(),
                            SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .expect("Time went backwards")
                                .as_secs(),
                        ));

                        format!(
                            "{}{}{}",
                            "{'success':true,'access_key':",
                            session_token.clone(),
                            "}"
                        ).to_owned()
                    } else {
                        "{'success':false}".to_owned()
                    };

                Response::builder()
                    .status(StatusCode::OK)
                    .body(Body::from(outgoing_json))
                    .map_err(|err| {
                        io::Error::new(
                            ErrorKind::Other,
                            format!("An error occured when building a response: {}", err),
                        )
                    })
            });

        Box::new(response)
    }

    /*
    // TODO: figure out how to merge this function with the above
    // TODO: how to correctly handle salting?
    // TODO: validate b2sum methods to be able to store salted hash in config
    fn check_password(&self, request: Request<Body>) -> ResponseFuture {
        let response = request
            .into_body()
            .concat2()
            .map_err(|err| {
                io::Error::new(
                    ErrorKind::Other,
                    format!("Failed to parse byte string: {}", err),
                )
            }).and_then(|body| {
                serde_json::from_slice(&body).map_err(|err| io::Error::new(ErrorKind::Other, err))
            }).and_then(move |request_body: PasswordRequest| {
                let password = &request_body.password.clone();

                // TODO: this should be read from config file
                let salted_hash: String = String::new();

                // TODO: this will be some string of 64 bytes encoded as hex
                let session_token: String = String::new();

                println!("{:?}", password);

                if check_password(password, salted_hash.as_str().as_bytes()) {
                    self.access_key = Some(Arc::new(Mutex::new(AccessKey::new(
                        session_token,
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .expect("Time went backwards")
                            .as_secs(),
                    ))));

                    // if password is correct, update self.{password_data}

                    Response::builder()
                        .status(StatusCode::OK)
                        .body(Body::from("Password received".to_owned()))
                        .map_err(|err| {
                            io::Error::new(
                                ErrorKind::Other,
                                format!("An error occured when building a response: {}", err),
                            )
                        })
                }
            });
        Box::new(response)
    }
    */

    fn serve_static_file(
        &self,
        root: PathBuf,
        valid_files: Vec<PathBuf>,
        mut request: Request<Body>,
    ) -> ResponseFuture {
        // TODO: set up a 404 page. Maybe hyper static file does it?

        let requested_path = &root.join(
            match PathBuf::from(&request.uri().path()).strip_prefix("/") {
                // strip_prefix(x) returns an error if the PathBuf does not
                // start with x. This situation should not occur in the web browser
                // but in case it does, we just return the path without removing
                // the initial "/"
                Ok(v) => v,
                Err(_err) => Path::new(request.uri().path()),
            },
        );

        if requested_path == &root {
            *request.uri_mut() = hyper::Uri::from_static("/index.html");
        } else if !valid_files.contains(&requested_path) {
            *request.uri_mut() = hyper::Uri::from_static("/404.html");
        }

        // resolve request and create response
        let response = hyper_staticfile::resolve(&root, &request)
            .map(move |result| {
                hyper_staticfile::ResponseBuilder::new()
                    .build(&request, result)
                    .map_err(|err| {
                        io::Error::new(
                            ErrorKind::Other,
                            format!("An error occured when building a response: {}", err),
                        )
                    })
            }).and_then(|response| future::result(response));
        Box::new(response)
    }

    fn route(&mut self, request: Request<Body>) -> ResponseFuture {
        let root_dir: PathBuf = self.static_dir.clone();
        let salted_hash: String = String::new();
        let valid_files: Vec<PathBuf> = match list_files(root_dir.clone()) {
            Ok(v) => v,
            Err(_err) => vec![PathBuf::from("404.html")],
        };

        match (request.method(), request.uri().path()) {
            (&Method::GET, "/api/v1/list_games") => self.list_games(),
            (&Method::POST, "/api/v1/start_game") => self.start_game(request),
            (&Method::POST, "/api/v1/check_password") => self.check_password(request, salted_hash),
            (&Method::GET, _) => self.serve_static_file(root_dir, valid_files, request),
            _ => self.invalid_endpoint(),
        }
    }
}
