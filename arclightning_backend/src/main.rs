extern crate futures;
extern crate hyper;
#[macro_use] 
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate toml;

use futures::future;
use hyper::rt::{Future, Stream};
use hyper::service::service_fn;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use std::path::PathBuf;

// after this concept is further understood, will switch to 'Either'
type ResponseFuture = Box<Future<Item = Response<Body>, Error = hyper::Error> + Send>;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Game {
    name: String,
    description: String,
    genre: Vec<String>,
    thumbnail_path: PathBuf,
    exe_path: PathBuf,
}

fn router(request: Request<Body>) -> ResponseFuture {

    let mut games_list: Vec<Game> = Vec::new();

    let mut response = Response::new(Body::empty());

    match (request.method(), request.uri().path()) {
        // TODO: get will send over games list
        (&Method::GET, "/api/v1/list_games") => {
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
        .map_err(|err| eprintln!("server error: {}", err));


    println!("Listening on http://{}", addr);
    hyper::rt::run(server);
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_read_toml() {

        use std::collections::HashMap;
        use std::fs::File;
        use std::io::Read;
        use serde_json::value::Value;

        // Read in a specific file
        let toml_filepath: PathBuf = ["test_files", "test_games.toml"].iter().collect(); 
        let mut games_toml = String::new();

        let mut file = File::open(&toml_filepath).expect("Could not find .toml file");


        file.read_to_string(&mut games_toml)
            .unwrap_or_else(|err| panic!("Error while reading .toml file: [{}]", err));

        let games: HashMap<String, Game> = toml::from_str(&games_toml).unwrap();
        println!("{:#?}", games);

        let mut test_games: HashMap<String, Game> = HashMap::new();
        test_games.insert("touhou_123".to_owned(),
            Game{
                name: "Touhou".to_owned(),
                description: "bullet hell with waifus".to_owned(),
                genre: vec!["bullet hell".to_owned(),
                            "anime".to_owned(),
                ],
                thumbnail_path: PathBuf::from(r"path\to\touhou\thumbnail"),
                exe_path: PathBuf::from(r"C:\Users\THISUSER\TOUHOU_PATH"),
        });

        test_games.insert("melty_blood".to_owned(),
            Game{
                name: "Melty Blood".to_owned(),
                description: "fighter with waifus".to_owned(),
                genre: vec!["fighter".to_owned(),
                            "anime".to_owned(),
                            "2d".to_owned(),
                ],
                thumbnail_path: PathBuf::from(r"path\to\melty_blood\thumbnail"),
                exe_path: PathBuf::from(r"C:\Users\THISUSER\MELTY_BLOOD_PATH"),
        });


        assert_eq!(games, test_games);
    }
}
