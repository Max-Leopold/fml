use std::fs;
use std::path::Path;

use anyhow::{anyhow, Error, Result};
use serde::{Deserialize, Serialize};

const CONFIG_DIR: &str = ".config";
const APP_DIR: &str = "fml";
const CONFIG_FILE: &str = "config.yml";
const DEFAULT_MODS_DIR_PATH: &str = "/opt/factorio/mods/";
const DEFAULT_SERVER_CONFIG_PATH: &str = "/opt/factorio/config/server-settings.json";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FmlConfig {
    pub mods_dir_path: String,
    pub server_config_path: String,
}

impl FmlConfig {
    pub fn load_config(&mut self) -> Result<()> {
        let config_path = match dirs::home_dir() {
            Some(path) => {
                let home = Path::new(&path);
                let config_dir = home.join(CONFIG_DIR);
                let app_dir = config_dir.join(APP_DIR);

                if !app_dir.exists() {
                    std::fs::create_dir_all(&app_dir)?;
                }

                app_dir.join(CONFIG_FILE)
            }
            None => return Err(anyhow!("Could not find home directory")),
        };

        if config_path.exists() {
            let config = std::fs::read_to_string(&config_path)?;
            let config_yml: FmlConfig = match serde_yaml::from_str(&config) {
                Ok(config) => config,
                Err(err) => return Err(anyhow!("Could not parse config file: {}", err)),
            };

            self.mods_dir_path = config_yml.mods_dir_path;
            self.server_config_path = config_yml.server_config_path;
        } else {
            println!("No config file found, creating one");
            println!("Config fill be saved to: {}", config_path.display());

            println!(
                "\nEnter path to mods folder (default: {}): ",
                DEFAULT_MODS_DIR_PATH
            );
            let mut mods_dir_path = String::new();
            std::io::stdin().read_line(&mut mods_dir_path)?;
            mods_dir_path = mods_dir_path.trim().to_string();
            if mods_dir_path.is_empty() {
                mods_dir_path = DEFAULT_MODS_DIR_PATH.to_string();
            }
            self.mods_dir_path = match fs::canonicalize(mods_dir_path) {
                Ok(path) => path.to_str().unwrap().to_string(),
                Err(_) => {
                    return Err(Error::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!(
                        "Could not find mods folder. You can use both absolute and relative paths."
                    ),
                    )))
                }
            };
            println!("Using mods folder at: {}", self.mods_dir_path);

            println!(
                "\nEnter path to server-settings.json (default: {}): ",
                DEFAULT_SERVER_CONFIG_PATH
            );
            let mut server_config_path = String::new();
            std::io::stdin().read_line(&mut server_config_path)?;
            server_config_path = server_config_path.trim().to_string();
            if server_config_path.is_empty() {
                server_config_path = DEFAULT_SERVER_CONFIG_PATH.to_string();
            }
            self.server_config_path =
                match fs::canonicalize(&server_config_path) {
                    Ok(path) => path.to_str().unwrap().to_string(),
                    Err(_) => return Err(Error::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!(
                            "Could not find server settings file. You can use both absolute and relative paths."
                        ),
                    ))),
                };
            println!("Using server settings file at: {}", self.server_config_path);

            let config = serde_yaml::to_string(&self)?;
            std::fs::write(&config_path, config)?;

            println!("Config file created");
        }

        Ok(())
    }
}
