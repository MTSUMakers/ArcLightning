use std::collections::HashMap;
use std::fs::File;
use std::io::{self, ErrorKind, Read};
use std::path::PathBuf;
use toml;

// using PartialEq for unit tests
// Using clone in a unit test atm.  Might not be necessary
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Game {
    name: String,
    description: String,
    genres: Vec<String>,
    thumbnail_path: PathBuf,
    #[serde(skip_serializing)]
    pub exe_path: PathBuf,
    #[serde(skip_serializing)]
    pub exe_args: Vec<String>,
}

pub fn toml_to_hashmap(toml_filepath: &PathBuf) -> Result<HashMap<String, Game>, io::Error> {
    let mut games_toml = String::new();
    let mut file = File::open(&toml_filepath)?;
    file.read_to_string(&mut games_toml)?;

    // error casting for homogeneous errors
    toml::from_str(&games_toml).map_err(|e| io::Error::new(ErrorKind::Other, e))
}
