/*
 * This will be a file that can be run with 2 arguments,
 * the new password and the config file.
 *
 * It will write the new password to the config file, where
 * the config file is completely rewritten with the new password
 *
 * Config file will need a password field as an Option<String>
 *
 * This will use the bcrypt crate.
 */

extern crate bcrypt;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate toml;

mod config;

use config::{unpack_toml, write_toml, Config, Game};
use std::io;
use std::path::PathBuf;

fn main() -> Result<(), io::Error> {
    // Note: This path requires that set_password be run from project root directory
    let toml_filepath: PathBuf = ["server_config.toml"].iter().collect();

    let mut config: Config = unpack_toml(&toml_filepath)?;

    config.set_password("catgirls".to_string());

    write_toml(&config, &toml_filepath)?;

    Ok(())
}
