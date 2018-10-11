extern crate futures;
extern crate hyper;
extern crate hyper_staticfile;
#[macro_use]
extern crate serde_derive;
extern crate rand;
extern crate serde_json;
extern crate toml;

mod game;
mod password;
mod router;
mod tests;

use futures::Future;
use game::{toml_to_hashmap, Game};
use hyper::Server;

use std::collections::HashMap;
use std::io::{self, ErrorKind};
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};

fn main() -> Result<(), io::Error> {
    // Read initial games toml config
    let toml_filepath: PathBuf = ["test_files", "test_games.toml"].iter().collect();

    // Store games locally on server
    let games: HashMap<String, Game> = toml_to_hashmap(&toml_filepath)?;

    // put the games data into the router struct
    let router = router::Router::new(games);

    // Host server
    let addr = ([127, 0, 0, 1], 3000).into();

    let server = Server::bind(&addr)
        .serve(router)
        .map_err(|err| eprintln!("server error: {}", err));

    println!("Listening on http://{}", addr);
    hyper::rt::run(server);
    Ok(())
}

