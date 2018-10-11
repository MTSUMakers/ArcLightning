use super::*;
use futures::{future, Stream};
use hyper::rt::Future;
use hyper::{Body, Error, Method, Request, Response, StatusCode};
use std::fs::read_dir;
use std::path::Path;

type ResponseFuture = Box<Future<Item = Response<Body>, Error = io::Error> + Send>;

fn visit_dirs(dir: PathBuf) -> Result<Vec<PathBuf>, io::Error> {
    let result = if dir.is_dir() {
        read_dir(dir)?
            .flatten()
            .map(|x| x.path())
            .flat_map(visit_dirs)
            .flatten()
            .collect()
    } else {
        vec![dir]
    };
    Ok(result)
}

#[derive(Debug, Clone)]
pub struct Router {
    games_list: Arc<Mutex<HashMap<String, Game>>>,
}

#[derive(Debug, Deserialize, Clone)]
struct StartGameRequest {
    id: String,
}

#[derive(Debug, Deserialize, Clone)]
struct ServeStaticFileRequest {
    target_file: String,
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
            games_list: self.games_list.clone(),
        }))
    }
}

impl Router {
    pub fn new(games_list: HashMap<String, Game>) -> Self {
        Router {
            games_list: Arc::new(Mutex::new(games_list)),
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
            .games_list
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
        let games_list = self.games_list.clone();

        let req_body = request.into_body();

        let response = req_body
            .concat2()
            .map_err(|err| {
                io::Error::new(
                    ErrorKind::Other,
                    format!("Failed to parse byte string: {}", err),
                )
            }).and_then(|body| {
                serde_json::from_slice(&body).map_err(|err| io::Error::new(ErrorKind::Other, err))
            }).and_then(move |request_body: StartGameRequest| {
                let games_list = games_list.lock().map_err(|_e| {
                    io::Error::new(
                        ErrorKind::Other,
                        "Failed to acquire mutex lock on games list".to_owned(),
                    )
                });

                games_list.map(|games_list| {
                    let game = games_list.get(&request_body.id).ok_or_else(|| {
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

    fn serve_static_file(
        &self,
        root: PathBuf,
        valid_files: Vec<PathBuf>,
        mut request: Request<Body>,
    ) -> ResponseFuture {
        // TODO: set up a 404 page. Maybe hyper static file does it?
        //
        // TODO: remove unwraps

        let requested_path = &root.join(
            PathBuf::from(&request.uri().path())
                .strip_prefix("/")
                .unwrap(),
        );

        if requested_path == &root || requested_path == &root.join(PathBuf::from(r"\\")) {
            *request.uri_mut() = "/index.html".parse().unwrap();
        } else if !valid_files.contains(&requested_path) {
            *request.uri_mut() = "/404.html".parse().unwrap();
        }

        // resolve request
        let resolve_future = hyper_staticfile::resolve(&root, &request);

        let response = resolve_future
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

    fn route(&self, request: Request<Body>) -> ResponseFuture {
        let root_dir: PathBuf = ["..", "arclightning_frontend"].iter().collect();
        let valid_files: Vec<PathBuf> = visit_dirs(root_dir.clone()).unwrap();

        match (request.method(), request.uri().path()) {
            (&Method::GET, "/api/v1/list_games") => self.list_games(),
            (&Method::POST, "/api/v1/start_game") => self.start_game(request),
            (&Method::GET, _) => self.serve_static_file(root_dir, valid_files, request),
            _ => self.invalid_endpoint(),
        }
    }
}
