extern crate futures;
extern crate hyper;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate toml;

use futures::{future, Stream};
use hyper::rt::Future;
use hyper::{Body, Error, Method, Request, Response, Server, StatusCode};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, ErrorKind, Read};
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};

// after this concept is further understood, will switch to 'Either'
type ResponseFuture = Box<Future<Item = Response<Body>, Error = io::Error> + Send>;

// using PartialEq for unit tests
// Using clone in a unit test atm.  Might not be necessary
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct Game {
    name: String,
    description: String,
    genres: Vec<String>,
    thumbnail_path: PathBuf,
    #[serde(skip_serializing)]
    exe_path: PathBuf,
    #[serde(skip_serializing)]
    exe_args: Vec<String>,
}

#[derive(Debug, Clone)]
struct Router {
    games_list: Arc<Mutex<HashMap<String, Game>>>,
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
            // TODO: is cloning necessary here?
            games_list: self.games_list.clone(),
        }))
    }
}

impl Router {
    fn new(games_list: HashMap<String, Game>) -> Self {
        Router {
            games_list: Arc::new(Mutex::new(games_list)),
        }
    }

    fn invalid_endpoint(&self) -> ResponseFuture {
        Box::new(future::result(
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("uwu 404 not foundu hiss".to_owned()))
                .map_err(|_e| {
                    io::Error::new(
                        ErrorKind::Other,
                        "Failed to acquire mutex lock on games list".to_owned(),
                    )
                }),
        ))
    }

    fn list_games(&self) -> ResponseFuture {
        let mut response = Response::new(Body::empty());
        let response_tuple = match self
            .games_list
            .lock()
            .map_err(|_e| {
                io::Error::new(
                    ErrorKind::Other,
                    "Failed to acquire mutex lock on games list".to_owned(),
                )
            })
            .and_then(|games| {
                serde_json::to_string(&*games).map_err(|err| io::Error::new(ErrorKind::Other, err))
            })
            .map(|body| Body::from(body))
        {
            Ok(v) => (v, StatusCode::OK),
            Err(_e) => (
                Body::from("Internal server error".to_owned()),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
        };
        *response.body_mut() = response_tuple.0;
        *response.status_mut() = response_tuple.1;

        Box::new(future::ok(response))
    }

    fn start_game(&self, req_body: Body) -> ResponseFuture {
        let games_list = self.games_list.clone();

        let response = req_body
            .concat2()
            .map_err(|err| {
                io::Error::new(
                    ErrorKind::Other,
                    format!("Failed to acquire mutex lock on games list: {}", err).to_owned(),
                )
            })
            .map(|body| String::from_utf8(body.to_vec()).unwrap())
            .and_then(move |game_id| {
                let games_list = games_list.lock().map_err(|_e| {
                    io::Error::new(
                        ErrorKind::Other,
                        "Failed to acquire mutex lock on games list".to_owned(),
                    )
                });

                games_list.map(|games_list| {
                    let game = games_list.get(&game_id).ok_or_else(|| {
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
                                format!("An error occured when building a response: {}", err)
                                    .to_owned(),
                            )
                        })
                })
            })
            .flatten();
        Box::new(response)
    }

    fn route(&self, request: Request<Body>) -> ResponseFuture {
        match (request.method(), request.uri().path()) {
            (&Method::GET, "/api/v1/list_games") => self.list_games(),
            (&Method::POST, "/api/v1/start_game") => self.start_game(request.into_body()),
            _ => self.invalid_endpoint(),
        }
    }
}

fn toml_to_hashmap(toml_filepath: PathBuf) -> Result<HashMap<String, Game>, io::Error> {
    let mut games_toml = String::new();
    let mut file = File::open(&toml_filepath)?;
    file.read_to_string(&mut games_toml)?;

    // error casting for homogeneous errors
    toml::from_str(&games_toml).map_err(|e| io::Error::new(ErrorKind::Other, e))
}

fn main() -> Result<(), io::Error> {
    // Read initial games toml config
    let toml_filepath: PathBuf = ["test_files", "test_games.toml"].iter().collect();

    // Store games locally on server
    let games: HashMap<String, Game> = toml_to_hashmap(toml_filepath)?;

    // put the games data into the router struct
    let router = Router::new(games);

    // Host server
    let addr = ([127, 0, 0, 1], 3000).into();

    let server = Server::bind(&addr)
        .serve(router)
        .map_err(|err| eprintln!("server error: {}", err));

    println!("Listening on http://{}", addr);
    hyper::rt::run(server);

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_json_serialization() {
        // Read in a specific file
        let toml_filepath: PathBuf = ["test_files", "test_games.toml"].iter().collect();
        let games: HashMap<String, Game> = toml_to_hashmap(toml_filepath).unwrap();

        // serialize as json
        let json_object_touhou = serde_json::to_string(&games.get("touhou_123")).unwrap();
        let json_object_melty_blood = serde_json::to_string(&games.get("melty_blood")).unwrap();

        // test cases separately to get around the nondeterministic order for hashmap
        let test_json_touhou = "{\"name\":\"Touhou\",\
                                \"description\":\"bullet hell with waifus\",\
                                \"genres\":[\"bullet hell\",\"anime\"],\
                                \"thumbnail_path\":\"path/to/touhou/thumbnail\"}";
        let test_json_mb = "{\"name\":\"Melty Blood\",\
                            \"description\":\"fighter with waifus\",\
                            \"genres\":[\"fighter\",\"anime\",\"2d\"],\
                            \"thumbnail_path\":\"path/to/melty_blood/thumbnail\"}";

        assert_eq!(json_object_touhou, test_json_touhou);
        assert_eq!(json_object_melty_blood, test_json_mb);
    }

    #[test]
    fn test_games_serialization() {
        // Read in a specific file
        let toml_filepath: PathBuf = ["test_files", "test_games.toml"].iter().collect();
        let games: HashMap<String, Game> = toml_to_hashmap(toml_filepath).unwrap();

        let games_clone = games.clone();

        // wrap all the games in a mutex
        // note that this moves games into the mutex
        let games_data = Arc::new(Mutex::new(games));

        assert_eq!(games_clone, *games_data.lock().unwrap());
    }

    #[test]
    fn test_read_toml() {
        // Read in a specific file
        let toml_filepath: PathBuf = ["test_files", "test_games.toml"].iter().collect();
        let games: HashMap<String, Game> = toml_to_hashmap(toml_filepath).unwrap();

        let mut test_games: HashMap<String, Game> = HashMap::new();
        test_games.insert(
            "touhou_123".to_owned(),
            Game {
                name: "Touhou".to_owned(),
                description: "bullet hell with waifus".to_owned(),
                genres: vec!["bullet hell".to_owned(), "anime".to_owned()],
                thumbnail_path: PathBuf::from(r"path\to\touhou\thumbnail"),
                exe_path: PathBuf::from(r"test_files\touhou_game.exe"),
                exe_args: vec!["arg1".to_owned(), "arg2".to_owned()],
            },
        );

        test_games.insert(
            "melty_blood".to_owned(),
            Game {
                name: "Melty Blood".to_owned(),
                description: "fighter with waifus".to_owned(),
                genres: vec!["fighter".to_owned(), "anime".to_owned(), "2d".to_owned()],
                thumbnail_path: PathBuf::from(r"path\to\melty_blood\thumbnail"),
                exe_path: PathBuf::from(r"test_files\melty_blood_game.exe"),
                exe_args: vec!["arg1".to_owned(), "arg2".to_owned()],
            },
        );
        assert_eq!(games, test_games);
    }
}
