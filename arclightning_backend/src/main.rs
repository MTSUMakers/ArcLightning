extern crate futures;
extern crate hyper;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde_json;
extern crate toml;

use futures::future;
use hyper::rt::{Future, Stream};
use hyper::service::service_fn;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use std::path::PathBuf;

// after this concept is further understood, will switch to 'Either'
type BoxFuture = Box<Future<Item = Response<Body>, Error = hyper::Error> + Send>;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Game {
    id: PathBuf,
    name: PathBuf,
    description: PathBuf,
    genre: PathBuf, // TODO: make a vector
    thumbnail_path: PathBuf,
    exe_path: PathBuf,
}

fn router(request: Request<Body>) -> BoxFuture {

    let mut games_list: Vec<Game> = Vec::new();

    let mut response = Response::new(Body::empty());

    match (request.method(), request.uri().path()) {
        // TODO: get will send over games list
        (&Method::GET, "/") => {
            // TODO: convert a vector of struct "Game" into a single json:
            *response.body_mut() = Body::from("games will go here");
        }
        // TODO: post will probably figure out which game to launch?
        (&Method::POST, "/echo") => {
            *response.body_mut() = request.into_body();
        }
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        }
    }

    Box::new(future::ok(response))
}

fn main() {
    let addr = ([127, 0, 0, 1], 3000).into();


    let server = Server::bind(&addr)
        .serve(|| service_fn(router))
        .map_err(|e| eprintln!("server error: {}", e));


    println!("Listening on http://{}", addr);
    hyper::rt::run(server);
}


mod test {
    use super::*;

    #[test]
    fn test_read_toml() {

        use std::fs::{File};
        use std::io::Read;
        use serde_json::value::Value;

        // Read in a specific file
        let toml_filepath = PathBuf::from(
            r"C:\Users\Sam\Documents\CSCI_4700\ArcLightning\test_files\test_games.toml");
        let mut games_toml = String::new();

        let mut file = match File::open(&toml_filepath) {
            Ok(file) => file,
            Err(_) => panic!("Could not find .toml file.")
        };


        file.read_to_string(&mut games_toml)
            .unwrap_or_else(|err| panic!("Error while reading .toml file: [{}]", err));

        let game: Game = match toml::from_str(&games_toml) {
            Ok(g) => g,
            Err(e) => panic!("Error while parsing file with toml: {}", e)
        };


        println!("{:#?}", game);

        let test_game = 
            Game{
                id: PathBuf::from("touhou_123"),
                name: PathBuf::from("Touhou"),
                description: PathBuf::from("bullet hell with waifus"),
                genre: PathBuf::from("bullet hell"),
                thumbnail_path: PathBuf::from(r"path\to\touhou\thumbnail"),
                exe_path: PathBuf::from(r"C:\Users\THISUSER\TOUHOU_PATH"),
        };

        assert_eq!(game, test_game);
    }
}
