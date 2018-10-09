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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_read_toml() {
        use serde_json::value::Value;
        use std::collections::HashMap;
        use std::fs::File;
        use std::io::Read;

        // Read in a specific file
        let toml_filepath: PathBuf = ["test_files", "test_games.toml"].iter().collect();
        let mut games_toml = String::new();

        let mut file = File::open(&toml_filepath).expect("Could not find .toml file");

        file.read_to_string(&mut games_toml)
            .unwrap_or_else(|err| panic!("Error while reading .toml file: [{}]", err));

        let games: HashMap<String, Game> = toml::from_str(&games_toml).unwrap();
        println!("{:#?}", games);

        let mut test_games: HashMap<String, Game> = HashMap::new();
        test_games.insert(
            "touhou_123".to_owned(),
            Game {
                name: "Touhou".to_owned(),
                description: "bullet hell with waifus".to_owned(),
                genre: vec!["bullet hell".to_owned(), "anime".to_owned()],
                thumbnail_path: PathBuf::from(r"path\to\touhou\thumbnail"),
                exe_path: PathBuf::from(r"C:\Users\THISUSER\TOUHOU_PATH"),
            },
        );

        test_games.insert(
            "melty_blood".to_owned(),
            Game {
                name: "Melty Blood".to_owned(),
                description: "fighter with waifus".to_owned(),
                genre: vec!["fighter".to_owned(), "anime".to_owned(), "2d".to_owned()],
                thumbnail_path: PathBuf::from(r"path\to\melty_blood\thumbnail"),
                exe_path: PathBuf::from(r"C:\Users\THISUSER\MELTY_BLOOD_PATH"),
            },
        );

        assert_eq!(games, test_games);
    }

}
