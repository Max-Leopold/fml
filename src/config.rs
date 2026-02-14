// Configuration — implemented in checkpoint 1.2
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FmlConfig {
    pub mods_dir_path: String,
    pub server_config_path: String,
}

impl FmlConfig {
    pub fn init() -> Result<()> {
        println!("Hello, FML!");
        Ok(())
    }

    pub fn load() -> Result<Self> {
        todo!()
    }
}
