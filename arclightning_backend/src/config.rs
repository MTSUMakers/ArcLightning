use bcrypt::{hash, verify};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, ErrorKind, Read, Write};
use std::path::PathBuf;
use toml;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Config {
    pub listen_port: u16,
    pub static_dir: PathBuf,
    pub password: Option<String>,
    pub games: HashMap<String, Game>,
}

impl Config {
    pub fn set_password(&mut self, password: &str) -> std::io::Result<()> {
        // The cost is set to 4 for our demo purposes to keep speed up
        let hashed_password = hash(&password, 4).map_err(|err| {
            io::Error::new(
                ErrorKind::Other,
                format!("An error occured when serializing config toml: {}", err),
            )
        })?;
        self.password = Some(hashed_password);
        Ok(())
    }

    pub fn write_to_path(&self, toml_filepath: &PathBuf) -> std::io::Result<()> {
        let mut file = File::create(toml_filepath)?;

        let toml_string = toml::to_string(&self).map_err(|err| {
            io::Error::new(
                ErrorKind::Other,
                format!("An error occured when serializing config toml: {}", err),
            )
        })?;

        file.write_all(toml_string.as_bytes())
    }
}

// using PartialEq for unit tests
// Using clone in a unit test atm.  Might not be necessary
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Game {
    pub name: String,
    pub description: String,
    pub genres: Vec<String>,
    pub thumbnail_path: PathBuf,
    pub exe_path: PathBuf,
    pub exe_args: Vec<String>,
}

pub fn load(toml_filepath: &PathBuf) -> Result<Config, io::Error> {
    let mut config_toml = String::new();
    File::open(&toml_filepath)?.read_to_string(&mut config_toml)?;

    // error casting for homogeneous errors
    toml::from_str(&config_toml).map_err(|err| io::Error::new(ErrorKind::Other, err))
}
