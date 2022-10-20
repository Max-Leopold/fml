use crate::factorio::api;
use crate::factorio::mods_config;
use crate::factorio::server_config;
use log::info;

#[derive(Default, Debug)]
pub struct FML {
    pub mods: Vec<api::Mod>,
    pub mods_config: mods_config::ModsConfig,
    pub server_config: server_config::ServerConfig,
}

impl FML {
    pub fn with_server_config(&mut self, server_config_path: &str) -> &mut Self {
        info!("Loading server config from {}", server_config_path);
        self.server_config = server_config::get_server_config(server_config_path).unwrap();
        self
    }

    pub fn with_mods_config(&mut self, mods_config_path: &str) -> &mut Self {
        info!("Loading mods config from {}", mods_config_path);
        self.mods_config = mods_config::get_mods_config(mods_config_path).unwrap();
        self
    }

    pub fn start(&mut self) -> String {
        info!("Starting FML");
        "Shutting down FML".to_string()
    }
}
