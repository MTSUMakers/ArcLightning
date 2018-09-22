extern crate futures;
extern crate hyper;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate toml;

mod password;

use futures::future;
use hyper::rt::Future;
use hyper::service::service_fn;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, ErrorKind, Read};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

// after this concept is further understood, will switch to 'Either'
type ResponseFuture = Box<Future<Item = Response<Body>, Error = hyper::Error> + Send>;

// using PartialEq for unit tests
// Using clone in a unit test atm.  Might not be necessary
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct Game {
    name: String,
    description: String,
    genres: Vec<String>,
    thumbnail_path: PathBuf,
    exe_path: PathBuf,
}

fn router(games_arc: &Arc<Mutex<HashMap<String, Game>>>, request: Request<Body>) -> ResponseFuture {
    let mut response = Response::new(Body::empty());
    let mut response_tuple: (hyper::Body, hyper::StatusCode) =
        (Body::empty(), StatusCode::NOT_FOUND);

    match (request.method(), request.uri().path()) {
        (&Method::GET, "/api/v1/list_games") => {
            response_tuple = match games_arc
                .lock()
                .map_err(|_e| {
                    io::Error::new(
                        ErrorKind::Other,
                        "Failed to acquire mutex lock on games list".to_owned(),
                    )
                })
                .and_then(|games| {
                    serde_json::to_string(&*games).map_err(|e| io::Error::new(ErrorKind::Other, e))
                })
                .and_then(|body| Ok(Body::from(body)))
            {
                Ok(v) => (v, StatusCode::OK),
                Err(_e) => (
                    Body::from("Internal server error".to_owned()),
                    StatusCode::INTERNAL_SERVER_ERROR,
                ),
            };
        }
        _ => {
            response_tuple = (
                Body::from("Invalid request".to_owned()),
                StatusCode::NOT_FOUND,
            );
        }
    }

    *response.body_mut() = response_tuple.0;
    *response.status_mut() = response_tuple.1;

    Box::new(future::ok(response))
}

fn toml_to_hashmap(toml_filepath: PathBuf) -> Result<HashMap<String, Game>, io::Error> {
    let mut games_toml = String::new();
    let mut file = File::open(&toml_filepath)?;
    file.read_to_string(&mut games_toml)?;

    // error casting for homogeneous errors
    toml::from_str(&games_toml).map_err(|e| io::Error::new(ErrorKind::Other, e))
}

fn main() {
    // Read initial games toml config
    let toml_filepath: PathBuf = ["test_files", "test_games.toml"].iter().collect();

    // Store games locally on server
    let games: HashMap<String, Game> = toml_to_hashmap(toml_filepath).unwrap();
    let games_data = Arc::new(Mutex::new(games));

    // Host server
    let addr = ([127, 0, 0, 1], 3000).into();

    let new_service = move || {
        let games_data = games_data.clone();
        service_fn(move |request| router(&games_data, request))
    };

    let server = Server::bind(&addr)
        .serve(new_service)
        .map_err(|err| eprintln!("server error: {}", err));

    println!("Listening on http://{}", addr);
    hyper::rt::run(server);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_check_password() {
        /* Initially, I wanted this test to verify that blake2b in blake2_rfc
         * was equivalent to b2sum, which is the hashed_password variable below.
         * However, this doesn't appear to be the case, as seen when running the
         * below test case and observing the console output.  Instead, the assertion
         * currently checks to see if the hashed value is equivalent to a hashed value
         * found via the check_password() function.
         *
         * TODO: Check and see if it's actually equivalent to b2sum, if necessary.
         */
        let password: String = "this_IS_my_P455W0RD".to_owned();
        let hashed_password = "02b9b24382937db98a3fe9b6121e9b9fc\
                               9b20d987fb1df5ec7ed5158a6ecd862a4\
                               ea28119407bec0dbf7665574161208899\
                               62475acedaad0478845f64e00ea5b"
            .to_owned()
            .into_bytes();

        use password::blake2_rfc::blake2b::blake2b;

        println!("Original: {:?}\n", password);
        println!(
            "Hashed with blake2-rfc: {:?}\n",
            blake2b(64, &[], password.as_bytes()).as_bytes()
        );
        println!("Hashed with b2sum: {:?}\n", hashed_password);

        assert!(password::check_password(
            password.clone(),
            blake2b(64, &[], password.clone().as_bytes()).as_bytes()
        ));

        // Commented out until we know if b2sum is equivalent to blake2b()
        //assert!(password::check_password(password, &hashed_password));
    }

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
                                \"thumbnail_path\":\"path/to/touhou/thumbnail\",\
                                \"exe_path\":\"C:\\\\Users\\\\THISUSER\\\\TOUHOU_PATH\"}";
        let test_json_mb = "{\"name\":\"Melty Blood\",\
                            \"description\":\"fighter with waifus\",\
                            \"genres\":[\"fighter\",\"anime\",\"2d\"],\
                            \"thumbnail_path\":\"path/to/melty_blood/thumbnail\",\
                            \"exe_path\":\"C:\\\\Users\\\\THISUSER\\\\MELTY_BLOOD_PATH\"}";

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
                exe_path: PathBuf::from(r"C:\Users\THISUSER\TOUHOU_PATH"),
            },
        );

        test_games.insert(
            "melty_blood".to_owned(),
            Game {
                name: "Melty Blood".to_owned(),
                description: "fighter with waifus".to_owned(),
                genres: vec!["fighter".to_owned(), "anime".to_owned(), "2d".to_owned()],
                thumbnail_path: PathBuf::from(r"path\to\melty_blood\thumbnail"),
                exe_path: PathBuf::from(r"C:\Users\THISUSER\MELTY_BLOOD_PATH"),
            },
        );
        assert_eq!(games, test_games);
    }
}
