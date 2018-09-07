extern crate futures;
extern crate hyper;
#[macro_use]
extern crate serde_json;

use futures::future;
use hyper::rt::{Future, Stream};
use hyper::service::service_fn;
use hyper::{Body, Method, Request, Response, Server, StatusCode};

// after this concept is further understood, will switch to 'Either'
type BoxFuture = Box<Future<Item = Response<Body>, Error = hyper::Error> + Send>;

struct Game {
    id: u8,
    name: String,
    description: String,
    genres: Vec<String>,
    thumbnail_path: String,
    exe_path: String,
}

//TODO: rename as the router
fn echo(req: Request<Body>) -> BoxFuture {

    let mut games_list: Vec<Game> = Vec::new();

    games_list.push(
        Game{
            id: 0,
            name: "Touhou".to_string(),
            description: "waifus shooting stuff".to_string(),
            genres: vec!["bullet hell".to_string()],
            thumbnail_path: "TEMP_THUMBNAIL_PATH".to_string(),
            exe_path: r"C:\\Users\THISUSER\RESTOFTHEPATH".to_string(),
    });

    games_list.push(
        Game{
            id: 1,
            name: "Melty Blood".to_string(),
            description: "waifus fighting stuff".to_string(),
            genres: vec!["anime".to_string(), "2d".to_string(), "fighter".to_string()],
            thumbnail_path: "TEMP_THUMBNAIL_PATH".to_string(),
            exe_path: r"C:\\Users\THISUSER\RESTOFTHEPATH".to_string(),
    });

    let mut response = Response::new(Body::empty());

    match (req.method(), req.uri().path()) {
        // TODO: get will send over games list
        (&Method::GET, "/") => {
            // TODO: convert a vector of struct "Game" into a single json:
            *response.body_mut() = Body::from(games_list);
        }
        // TODO: post will probably figure out which game to launch?
        (&Method::POST, "/echo") => {
            *response.body_mut() = req.into_body();
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
        .serve(|| service_fn(echo))
        .map_err(|e| eprintln!("server error: {}", e));


    println!("Listening on http://{}", addr);
    hyper::rt::run(server);
}
