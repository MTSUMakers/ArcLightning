use std::collections::HashMap;
use std::fs::File;
use std::io::{self, ErrorKind, Read};
use std::path::PathBuf;
use toml;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Config {
    pub listen_port: u16,
    pub static_dir: PathBuf,
    pub games: HashMap<String, Game>,
}

// using PartialEq for unit tests
// Using clone in a unit test atm.  Might not be necessary
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Game {
    pub name: String,
    pub description: String,
    pub genres: Vec<String>,
    pub thumbnail_path: PathBuf,
    #[serde(skip_serializing)]
    pub exe_path: PathBuf,
    #[serde(skip_serializing)]
    pub exe_args: Vec<String>,
}

pub fn unpack_toml(toml_filepath: &PathBuf) -> Result<Config, io::Error> {
    let mut config_toml = String::new();
    File::open(&toml_filepath)?.read_to_string(&mut config_toml)?;

    // error casting for homogeneous errors
    toml::from_str(&config_toml).map_err(|err| io::Error::new(ErrorKind::Other, err))
}
