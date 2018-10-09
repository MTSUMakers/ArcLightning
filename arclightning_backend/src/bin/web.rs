use super::*;
use futures::{future, Stream};
use hyper::rt::Future;
use hyper::{Body, Error, Method, Request, Response, StatusCode};

type ResponseFuture = Box<Future<Item = Response<Body>, Error = io::Error> + Send>;

#[derive(Debug, Clone)]
pub struct WebRouter {
    games_list: Arc<Mutex<HashMap<String, Game>>>,
}

#[derive(Debug, Deserialize, Clone)]
struct StartGameRequest {
    id: String,
}

impl hyper::service::Service for WebRouter {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = io::Error;
    type Future = ResponseFuture;
    fn call(&mut self, req: Request<Self::ReqBody>) -> Self::Future {
        self.route(req)
    }
}

impl hyper::service::NewService for WebRouter {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = io::Error;
    type Service = WebRouter;
    type Future = Box<Future<Item = Self::Service, Error = Self::InitError> + Send>;
    type InitError = Error;
    fn new_service(&self) -> Self::Future {
        Box::new(future::ok(Self {
            games_list: self.games_list.clone(),
        }))
    }
}

impl WebRouter {
    pub fn new(games_list: HashMap<String, Game>) -> Self {
        WebRouter {
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
                        format!("An error occured when constructing 404 error: {}", err)
                    )
                }),
        ))
    }

    // TODO: implement these functions

    fn register(&self, req_body: Body) -> ResponseFuture {
        let games_list = self.games_list.clone();

        // TODO: parse body into username and password
        let response = req_body
            .concat2()
            .map_err(|err| {
                io::Error::new(
                    ErrorKind::Other,
                    format!("Failed to parse byte string: {}", err)
                )
            }).flatten();
        Box::new(response)
    }

    fn signin(&self, req_body: Body) -> ResponseFuture {
        let games_list = self.games_list.clone();

        // TODO: parse body into username and password
        let response = req_body
            .concat2()
            .map_err(|err| {
                io::Error::new(
                    ErrorKind::Other,
                    format!("Failed to parse byte string: {}", err)
                )
            }).flatten();
        Box::new(response)
    }

    fn check_in(&self, req_body: Body) -> ResponseFuture {
        let games_list = self.games_list.clone();

        // TODO: parse body into key
        let response = req_body
            .concat2()
            .map_err(|err| {
                io::Error::new(
                    ErrorKind::Other,
                    format!("Failed to parse byte string: {}", err)
                )
            }).flatten();
        Box::new(response)
    }

    fn check_out(&self, req_body: Body) -> ResponseFuture {
        let games_list = self.games_list.clone();

        // TODO: parse body into key
        let response = req_body
            .concat2()
            .map_err(|err| {
                io::Error::new(
                    ErrorKind::Other,
                    format!("Failed to parse byte string: {}", err)
                )
            }).flatten();
        Box::new(response)
    }

    fn check_settings(&self, req_body: Body) -> ResponseFuture {
        let games_list = self.games_list.clone();

        // TODO: parse body into key
        let response = req_body
            .concat2()
            .map_err(|err| {
                io::Error::new(
                    ErrorKind::Other,
                    format!("Failed to parse byte string: {}", err)
                )
            }).flatten();
        Box::new(response)
    }

    fn route(&self, request: Request<Body>) -> ResponseFuture {
        match (request.method(), request.uri().path()) {
            (&Method::POST, "/api/v1/register") => self.register(request.into_body()),
            (&Method::POST, "/api/v1/signin") => self.signin(request.into_body()),
            (&Method::POST, "/api/v1/check_in") => self.check_in(request.into_body()),
            (&Method::POST, "/api/v1/check_out") => self.check_out(request.into_body()),
            (&Method::POST, "/api/v1/check_settings") => self.check_settings(request.into_body()),
            _ => self.invalid_endpoint(),
        }
    }
}
