// TODO: expire cookie after 12 hours
// TODO: validate that cookie is sent to frontend correctly

use bcrypt::verify;
use config::{Config, Game};
use futures::{future, Stream};
use hyper::header::{COOKIE, LOCATION, SET_COOKIE};
use hyper::rt::Future;
use hyper::{Body, Error, Method, Request, Response, StatusCode};
use rand::Rng;
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
    password: String,
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
            access_key: access_key,
            access_time,
        }
    }
    pub fn dummy() -> Self {
        AccessKey {
            access_key: "failure_key".to_owned(),
            access_time: 0u64,
        }
    }
}

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
            password: self.password.clone(),
        }))
    }
}

impl Router {
    pub fn new(config: Config) -> Self {
        Router {
            games: Arc::new(Mutex::new(config.games)),
            static_dir: config.static_dir,
            access_key: Arc::new(Mutex::new(None)),
            password: config.password.unwrap_or_else(|| "".to_string()),
        }
    }

    fn invalid_endpoint(&self, root: &PathBuf, mut request: Request<Body>) -> ResponseFuture {
        // In case of an invalid endpoint, serve the static 404.html page
        *request.uri_mut() = hyper::Uri::from_static("/404.html");

        let response = hyper_staticfile::resolve(&root, &request)
            .map(move |result| {
                hyper_staticfile::ResponseBuilder::new()
                    .build(&request, result)
                    .map_err(|err| {
                        io::Error::new(
                            ErrorKind::Other,
                            format!(
                                "An error occured when constructing 404 error\
                                 after invalid endpoint: {}",
                                err
                            ),
                        )
                    })
            })
            .and_then(future::result);

        Box::new(response)
    }

    fn redirect_endpoint(&self) -> ResponseFuture {
        Box::new(future::result(
            Response::builder()
                .status(StatusCode::TEMPORARY_REDIRECT)
                .header(LOCATION, "/demonstration.html")
                .body(Body::empty())
                .map_err(|err| {
                    io::Error::new(
                        ErrorKind::Other,
                        format!(
                            "An error occured when redirecting to\
                             /demonstration.html: {}",
                            err
                        ),
                    )
                }),
        ))
    }

    fn api_fail(&self) -> ResponseFuture {
        Box::new(future::result(
            Response::builder()
                .status(StatusCode::FORBIDDEN)
                .body(Body::from(r#"{"success": false}"#))
                .map_err(|err| {
                    io::Error::new(
                        ErrorKind::Other,
                        format!(
                            "An error occured when accessing \
                             invalid api endpoint: {}",
                            err
                        ),
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
            })
            .and_then(|games| {
                serde_json::to_string(&*games).map_err(|err| io::Error::new(ErrorKind::Other, err))
            })
            .map(Body::from)
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
            })
            .and_then(|body| {
                serde_json::from_slice(&body).map_err(|err| io::Error::new(ErrorKind::Other, err))
            })
            .and_then(move |request_body: StartGameRequest| {
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

                    /*
                     *
                     * Skipping additional arguments for now
                     */

                    //let exe_args = game.exe_args.clone();
                    //Command::new(exe_path).args(exe_args).spawn()?;

                    println!("Starting game: {}", request_body.id);
                    Command::new(exe_path.clone())
                        .current_dir(exe_path.parent().unwrap())
                        .spawn()?;

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
            })
            .flatten();
        Box::new(response)
    }

    // Checks password at demo screen
    // If correct, returns serialized access key in the ResponseFuture
    fn check_password(
        &mut self,
        request: Request<Body>,
        hashed_password: String,
    ) -> ResponseFuture {
        let access_key = self.access_key.clone();

        let response = request
            .into_body()
            .concat2()
            .map_err(|err| {
                io::Error::new(
                    ErrorKind::Other,
                    format!("Failed to parse byte string: {}", err),
                )
            })
            .and_then(|body| {
                serde_json::from_slice(&body).map_err(|err| io::Error::new(ErrorKind::Other, err))
            })
            .and_then(move |request_body: PasswordRequest| {
                let password = request_body.password.clone();

                // 64 random bytes encoded as hex and stored as a string
                let mut session_token = [0u8; 64];
                rand::thread_rng().fill(&mut session_token[..]);
                let session_token = hex::encode(&session_token[..]);

                let outgoing_json: String =
                    if verify(&password, &hashed_password).map_err(|err| {
                        io::Error::new(
                            ErrorKind::Other,
                            format!("Failed to parse BcryptResult: {}", err),
                        )
                    })? {
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
                                .map_err(|err| {
                                    io::Error::new(
                                        ErrorKind::Other,
                                        format!("Time went backwards: {}", err),
                                    )
                                })?
                                .as_secs(),
                        ));
                        r#"{"success":true}"#.to_owned()
                    } else {
                        r#"{"success":false}"#.to_owned()
                    };

                Response::builder()
                    .status(StatusCode::OK)
                    .header(SET_COOKIE, session_token.clone())
                    .header("sam-token", session_token)
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

    fn check_header(&self, request: &Request<Body>) -> Result<bool, io::Error> {
        let access_key: String = self
            .access_key
            .lock()
            .map_err(|err| {
                io::Error::new(
                    ErrorKind::Other,
                    format!("Failed to acquire mutex on access key: {}", err),
                )
            })?
            .clone()
            .unwrap_or_else(AccessKey::dummy)
            .access_key;

        request
            .headers()
            .clone()
            .get(COOKIE)
            .ok_or_else(|| {
                io::Error::new(ErrorKind::Other, "Failed to acquire cookie from header")
            })?
            .to_str()
            .map_err(|err| {
                io::Error::new(
                    ErrorKind::Other,
                    format!("Failed to convert cookie to string: {}", err),
                )
            })?
            .to_string()
            .split("=")
            .skip(1)
            .next()
            .map(|cookie| cookie == access_key)
            .ok_or_else(|| io::Error::new(ErrorKind::Other, "Failed to acquire cookie from header"))
    }

    fn serve_static_file(
        &self,
        root: &PathBuf,
        valid_files: &[PathBuf],
        mut request: Request<Body>,
    ) -> ResponseFuture {
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

        if requested_path == root {
            *request.uri_mut() = hyper::Uri::from_static("/start.html");
        } else if !valid_files.contains(&requested_path) {
            *request.uri_mut() = hyper::Uri::from_static("/404.html");
        }

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
            })
            .and_then(future::result);

        Box::new(response)
    }

    fn route(&mut self, request: Request<Body>) -> ResponseFuture {
        let root_dir: PathBuf = self.static_dir.clone();
        let salted_hash: String = self.password.clone();
        let valid_files: Vec<PathBuf> = match list_files(root_dir.clone()) {
            Ok(v) => v,
            Err(_err) => vec![PathBuf::from("404.html")],
        };

        let correct_cookie: bool = self.check_header(&request).unwrap_or_else(|err| {
            //println!("{}", err);
            false
        });

        println!(
            "{} {} {}",
            request.method(),
            request.uri().path(),
            correct_cookie
        );

        match (request.method(), request.uri().path(), correct_cookie) {
            (&Method::GET, "/api/v1/list_games", true) => self.list_games(),
            (&Method::GET, "/api/v1/list_games", false) => self.api_fail(),

            (&Method::POST, "/api/v1/start_game", true) => self.start_game(request),
            (&Method::POST, "/api/v1/start_game", false) => self.api_fail(),

            (&Method::POST, "/api/v1/check_password", _) => {
                self.check_password(request, salted_hash)
            }

            (&Method::GET, "/games.html", false) => self.redirect_endpoint(),

            (&Method::GET, _, _) => self.serve_static_file(&root_dir, &valid_files, request),

            _ => self.invalid_endpoint(&root_dir, request),
        }
    }
}
