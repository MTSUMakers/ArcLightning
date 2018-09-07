extern crate futures;
extern crate hyper;
#[macro_use]
extern crate serde_json;

use futures::future;
use hyper::rt::{Future, Stream};
use hyper::service::service_fn;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use std::path::PathBuf;

// after this concept is further understood, will switch to 'Either'
type BoxFuture = Box<Future<Item = Response<Body>, Error = hyper::Error> + Send>;

struct Game {
    id: u8,
    name: PathBuf,
    description: PathBuf,
    genres: Vec<PathBuf>,
    thumbnail_path: PathBuf,
    exe_path: PathBuf,
}

fn router(request: Request<Body>) -> BoxFuture {

    let mut games_list: Vec<Game> = Vec::new();

    games_list.push(
        Game{
            id: 0,
            name: PathBuf::from("Touhou"),
            description: PathBuf::from("waifus shooting stuff"),
            genres: vec![PathBuf::from("bullet hell")],
            thumbnail_path: PathBuf::from("TEMP_THUMBNAIL_PATH"),
            exe_path: PathBuf::from(r"C:\\Users\THISUSER\RESTOFTHEPATH"),
    });

    games_list.push(
        Game{
            id: 1,
            name: PathBuf::from("Melty Blood"),
            description: PathBuf::from("waifus fighting stuff"),
            genres: vec![PathBuf::from("anime"), 
                         PathBuf::from("2d"),
                         PathBuf::from("fighter")],
            thumbnail_path: PathBuf::from("TEMP_THUMBNAIL_PATH"),
            exe_path: PathBuf::from(r"C:\\Users\THISUSER\RESTOFTHEPATH"),
    });

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
