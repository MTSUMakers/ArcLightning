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

    use std::fs::read_dir;
    let valid_files = list_files(path).unwrap();
    //println!("Valid files {:#?}", valid_files);

    let toml_path: PathBuf = [
        r"C:\\",
        "Users",
        "Sam",
        "Documents",
        "CSCI_4700",
        "ArcLightning",
        "arclightning_backend",
        "asdf.toml",
    ]
        .iter()
        .collect();

    //println!("Target file {:#?}", toml_path);

    assert!(valid_files.contains(&toml_path));
}

#[test]
fn test_read_toml() {
    // Read in a specific file
    //let toml_filepath: PathBuf = ["asdf.toml"].iter().collect();
    let toml_filepath: PathBuf = [
        r"C:\\",
        "Users",
        "Sam",
        "Documents",
        "CSCI_4700",
        "ArcLightning",
        "arclightning_backend",
        "asdf.toml",
    ]
        .iter()
        .collect();
    let config: Config = unpack_toml(&toml_filepath).unwrap();
    println!("{:#?}", config);
    let games = config.games_config;

    //println!("{:#?}", games);
    //let games: HashMap<String, Game> = toml_to_hashmap(&toml_filepath).unwrap();

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
    let toml_filepath: PathBuf = ["test_files", "server_config.toml"].iter().collect();
    let games: HashMap<String, Game> = toml_to_hashmap(&toml_filepath).unwrap();

    // serialize as json
    let json_object_touhou = serde_json::to_string(&games.get("game.touhou_123")).unwrap();
    let json_object_melty_blood = serde_json::to_string(&games.get("game.melty_blood")).unwrap();

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
    let toml_filepath: PathBuf = ["test_files", "server_config.toml"].iter().collect();
    let games: HashMap<String, Game> = toml_to_hashmap(&toml_filepath).unwrap();

    let games_clone = games.clone();

    // wrap all the games in a mutex
    // note that this moves games into the mutex
    let games_data = Arc::new(Mutex::new(games));

    assert_eq!(games_clone, *games_data.lock().unwrap());
}
