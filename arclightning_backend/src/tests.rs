#![cfg(test)]
use super::*;
use router::list_files;
use std::sync::{Arc, Mutex};

#[test]
fn test_paths() {
    // make sure we're in the right directory
    use std::env;
    let path = env::current_dir().unwrap();
    println!("The current directory is {}", path.display());

    let valid_files = list_files(path).unwrap();

    let toml_filepath: PathBuf = ["server_config.toml"].iter().collect();

    println!("Toml filepath: {:?}", toml_filepath);

    // Kind of sloppy for a test, but the previous list_files() takes ownership of path.
    // So we need to redefine it once more
    let path = env::current_dir().unwrap();
    let joined_path = path.join(&toml_filepath);

    println!("Joined toml filepath: {:?}", joined_path);

    assert!(valid_files.contains(&joined_path));
}

#[test]
fn test_read_toml() {
    // Read in a specific file
    let toml_filepath: PathBuf = ["server_config.toml"].iter().collect();
    let config: Config = Config::load(&toml_filepath).unwrap();
    println!("{:#?}", config);
    let games = config.games;

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

#[test]
fn test_json_serialization() {
    // Read in a specific file
    let toml_filepath: PathBuf = ["server_config.toml"].iter().collect();
    let config: Config = Config::load(&toml_filepath).unwrap();
    println!("{:#?}", config);
    let games = config.games;

    // serialize as json
    let json_object_touhou = serde_json::to_string(&games.get("touhou_123")).unwrap();
    let json_object_melty_blood = serde_json::to_string(&games.get("melty_blood")).unwrap();

    // test cases separately to get around the nondeterministic order for hashmap
    let test_json_touhou = "{\"name\":\"Touhou\",\
                            \"description\":\"bullet hell with waifus\",\
                            \"genres\":[\"bullet hell\",\"anime\"],\
                            \"thumbnail_path\":\"path/to/touhou/thumbnail\",\
                            \"exe_path\":\"test_files\\\\touhou_game.exe\",\
                            \"exe_args\":[\"arg1\",\"arg2\"]}";
    let test_json_mb = "{\"name\":\"Melty Blood\",\
                        \"description\":\"fighter with waifus\",\
                        \"genres\":[\"fighter\",\"anime\",\"2d\"],\
                        \"thumbnail_path\":\"path/to/melty_blood/thumbnail\",\
                        \"exe_path\":\"test_files\\\\melty_blood_game.exe\",\
                        \"exe_args\":[\"arg1\",\"arg2\"]}";

    assert_eq!(json_object_touhou, test_json_touhou);
    assert_eq!(json_object_melty_blood, test_json_mb);
}

#[test]
fn test_games_serialization() {
    // Read in a specific file
    let toml_filepath: PathBuf = ["server_config.toml"].iter().collect();
    let config: Config = Config::load(&toml_filepath).unwrap();
    println!("{:#?}", config);
    let games = config.games;

    let games_clone = games.clone();

    // wrap all the games in a mutex
    // note that this moves games into the mutex
    let games_data = Arc::new(Mutex::new(games));

    assert_eq!(games_clone, *games_data.lock().unwrap());
}

#[test]
fn test_check_password() {
    // this still fails...?
    let password: String = "this_IS my_P455W0RD!%".to_owned();
    let hashed_password: Vec<u8> = "a50f985ce10f2dfbf71e119ae69\
                                    522754b65e022c558d2ce9160df\
                                    4113060eb66bf5de6e1ce400c05\
                                    34a08db6916f4c2751353de29f8\
                                    4608dd0ebe67e57e12e0"
        .to_owned()
        .into_bytes();

    println!("{:?}", hashed_password);

    assert!(password::check_password(&password, &hashed_password));
}

#[test]
fn test_hexdump() {
    use rand::Rng;

    let mut session_token = [0u8; 64];
    rand::thread_rng().fill(&mut session_token[..]);
    let session_token = hex::encode(&session_token[..]);

    println!("{}", session_token);
    assert_eq!(hex::decode(session_token).unwrap().len(), 64);
}
