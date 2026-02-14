use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::path::Path;

const CONFIG_FILE: &str = "fml.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct FmlConfig {
    pub mods_dir_path: String,
    pub server_config_path: String,
}

impl FmlConfig {
    pub fn init() -> Result<()> {
        let mods_dir = prompt("Path to Factorio mods directory")?;
        let mods_path = Path::new(&mods_dir);
        if !mods_path.is_dir() {
            bail!("Mods directory does not exist: {}", mods_dir);
        }

        let server_config = prompt("Path to server-settings.json")?;
        let server_path = Path::new(&server_config);
        if !server_path.is_file() {
            bail!("Server settings file does not exist: {}", server_config);
        }

        let config = FmlConfig {
            mods_dir_path: mods_path
                .canonicalize()
                .context("Failed to canonicalize mods directory path")?
                .to_string_lossy()
                .into_owned(),
            server_config_path: server_path
                .canonicalize()
                .context("Failed to canonicalize server config path")?
                .to_string_lossy()
                .into_owned(),
        };

        let json = serde_json::to_string_pretty(&config)?;
        std::fs::write(CONFIG_FILE, json).context("Failed to write fml.json")?;
        println!("Configuration saved to {}", CONFIG_FILE);
        Ok(())
    }

    pub fn load() -> Result<Self> {
        let path = Path::new(CONFIG_FILE);
        if !path.exists() {
            bail!(
                "No fml.json found in current directory. Run `fml init` first."
            );
        }
        let contents = std::fs::read_to_string(path).context("Failed to read fml.json")?;
        let config: FmlConfig =
            serde_json::from_str(&contents).context("Failed to parse fml.json")?;
        Ok(config)
    }
}

fn prompt(message: &str) -> Result<String> {
    print!("{}: ", message);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim().to_string();
    if trimmed.is_empty() {
        bail!("Input cannot be empty");
    }
    Ok(trimmed)
}
