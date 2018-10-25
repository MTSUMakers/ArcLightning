extern crate futures;
extern crate hyper;
extern crate hyper_staticfile;
#[macro_use]
extern crate serde_derive;
extern crate rand;
extern crate serde_json;
extern crate toml;

mod config;
mod password;
mod router;
mod tests;

use config::{unpack_toml, Config, Game};
use futures::Future;
use hyper::Server;
use password::*;

use std::collections::HashMap;
use std::io;
use std::path::PathBuf;

fn main() -> Result<(), io::Error> {
    // Read initial games toml config
    let toml_filepath: PathBuf = ["server_config.toml"].iter().collect();

    // Unpack config
    let config: Config = unpack_toml(&toml_filepath)?;
    let games: HashMap<String, Game> = config.games_config;
    let listen_port: u16 = config.server_config.listen_port;
    let static_dir: PathBuf = config.server_config.static_dir;

    println!("Using assets directory: {:?}", static_dir);

    // put the games data into the router struct
    let router = router::Router::new(games, static_dir);

    // Host server
    let addr = ([127, 0, 0, 1], listen_port).into();

    let server = Server::bind(&addr)
        .serve(router)
        .map_err(|err| eprintln!("server error: {}", err));

    println!("Listening on http://{}", addr);
    hyper::rt::run(server);
    Ok(())
}
