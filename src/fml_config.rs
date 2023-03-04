use std::{env, fs};

use anyhow::{anyhow, Error, Result};
use serde::{Deserialize, Serialize};

const CONFIG_FILE: &str = "fml.json";
const DEFAULT_MODS_DIR_PATH: &str = "/opt/factorio/mods/";
const DEFAULT_SERVER_CONFIG_PATH: &str = "/opt/factorio/config/server-settings.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct FmlConfig {
    pub mods_dir_path: String,
    pub server_config_path: String,
    mods: Vec<Mod>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Mod {
    pub name: String,
    pub version: String,
}

fn read_line() -> String {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn read_line_with_default(default: &str) -> String {
    let input = read_line();
    if input.is_empty() {
        default.to_string()
    } else {
        input
    }
}

fn canonicalize_path(path: &str) -> Result<String, Error> {
    match fs::canonicalize(path) {
        Ok(path) => Ok(path.to_str().unwrap().to_string()),
        Err(_) => {
            return Err(Error::from(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Could not find path: {}", path),
            )))
        }
    }
}

impl FmlConfig {
    pub fn load_config() -> Result<FmlConfig, Error> {
        let current_working_dir = env::current_dir()?;
        let config_path = current_working_dir.join(CONFIG_FILE);

        if config_path.exists() {
            let config = std::fs::read_to_string(&config_path)?;
            match serde_json::from_str(&config) {
                Ok(config) => Ok(config),
                Err(err) => Err(anyhow!("Could not parse config file: {}", err)),
            }
        } else {
            Err(anyhow!(
                "No config file found. Please run `fml init` first."
            ))
        }
    }

    pub fn init() -> Result<FmlConfig, Error> {
        let current_working_dir = env::current_dir()?;
        let config_path = current_working_dir.join(CONFIG_FILE);
        if config_path.exists() {
            println!("Config file already exists");
            println!("Do you want to overwrite it? (y/n)");
            let input = read_line();
            if input != "y" {
                std::process::exit(0);
            } else {
                println!("Overwriting config file\n");
            }
        }

        println!(
            "\nWhere should FML store the mods? (default: {})",
            DEFAULT_MODS_DIR_PATH
        );
        let mut mods_dir_path = read_line_with_default(DEFAULT_MODS_DIR_PATH);
        mods_dir_path = canonicalize_path(&mods_dir_path)?;
        println!("Using mods directory: {}", mods_dir_path);

        println!(
            "\nWhere is the server config file? (default: {})",
            DEFAULT_SERVER_CONFIG_PATH
        );
        let mut server_config_path = read_line_with_default(DEFAULT_SERVER_CONFIG_PATH);
        server_config_path = canonicalize_path(&server_config_path)?;
        println!("Using server config file: {}", server_config_path);

        let config = FmlConfig {
            mods_dir_path,
            server_config_path,
            mods: Vec::new(),
        };

        config.save()?;

        Ok(config)
    }

    pub fn add_mod(&mut self, name: &str, version: &str) -> Result<(), Error> {
        self.mods.push(Mod {
            name: name.to_string(),
            version: version.to_string(),
        });

        self.save()
    }

    pub fn remove_mod(&mut self, name: &str) -> Result<(), Error> {
        self.mods.retain(|mod_| mod_.name != name);

        self.save()
    }

    fn save(&self) -> Result<(), Error> {
        let current_working_dir = env::current_dir()?;
        let config_path = current_working_dir.join(CONFIG_FILE);

        let config_str = serde_json::to_string_pretty(&self)?;
        std::fs::write(&config_path, config_str)?;

        Ok(())
    }
}
