use std::sync::{Arc, Mutex};

use crate::factorio::{self, api, installed_mods, mod_list, server_settings};
use crate::fml_config::FmlConfig;
use tui::style::{Color, Style};

use super::event::Events;
use super::install_mod_list::{InstallModItem, InstallModList};
use super::manage_mod_list::ManageModList;
use super::mod_downloader::{ModDownloadRequest, ModDownloader};

#[derive(Debug, Clone, Copy)]
pub enum Tab {
    Manage,
    Install,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActiveBlock {
    InstallModList,
    InstallSearch,
    InstallModDetails,
    ManageModList,
    QuitPopup,
}

pub struct Route {
    active_block: ActiveBlock,
    tab: Tab,
}

const DEFAULT_ROUTE: Route = Route {
    active_block: ActiveBlock::ManageModList,
    tab: Tab::Manage,
};

pub struct FML {
    pub install_mod_list: Arc<Mutex<InstallModList>>,
    pub manage_mod_list: Arc<Mutex<ManageModList>>,
    pub server_settings: server_settings::ServerSettings,
    pub fml_config: FmlConfig,
    pub events: Events,
    pub mod_downloader: ModDownloader,
    navigation_history: Vec<Route>,
    pub ticks: u64,
    pub should_quit: bool,
    pub scroll_offset: u16,
}

impl FML {
    pub fn new(fml_config: FmlConfig) -> FML {
        let server_settings =
            server_settings::get_server_settings(&fml_config.server_config_path).unwrap();
        let install_mod_list = Arc::new(Mutex::new(InstallModList::default()));
        let manage_mod_list = Arc::new(Mutex::new(ManageModList::default()));
        let mod_downloader = ModDownloader::new(install_mod_list.clone(), manage_mod_list.clone());
        let ticks = 0;
        let should_quit = false;
        let navigation_history = vec![DEFAULT_ROUTE];
        let scroll_offset = 0;
        let events = Events::with_config(None);

        let fml = FML {
            install_mod_list,
            manage_mod_list,
            server_settings,
            fml_config,
            events,
            mod_downloader,
            navigation_history,
            ticks,
            should_quit,
            scroll_offset,
        };

        fml.async_init();

        return fml;
    }

    fn async_init(&self) {
        let mods_dir_path = self.fml_config.mods_dir_path.clone();
        let install_mod_list = self.install_mod_list.clone();
        tokio::spawn(async move {
            // Load mod identifiers in registry
            let mod_identifiers = factorio::api::registry::Registry::mod_identifiers().await;
            match mod_identifiers {
                Ok(mod_identifiers) => {
                    let installed_mods = match installed_mods::read_installed_mods(&mods_dir_path) {
                        Ok(mods) => mods,
                        Err(e) => {
                            log::error!("Error reading installed mods: {}", e);
                            vec![]
                        }
                    };
                    let installed_mods = installed_mods
                        .into_iter()
                        .map(|mod_| (mod_.name.clone(), mod_))
                        .collect::<std::collections::HashMap<String, installed_mods::InstalledMod>>(
                        );

                    let mod_list_items = mod_identifiers
                        .into_iter()
                        .map(|mod_identifier| {
                            let mut mod_item = InstallModItem::new(mod_identifier.clone());
                            if installed_mods.contains_key(&mod_item.mod_identifier.name) {
                                mod_item.download_info.downloaded = true;
                            }
                            mod_item
                        })
                        .collect();

                    install_mod_list.lock().unwrap().set_items(mod_list_items);
                }
                Err(e) => {
                    // We should retry loading the mod identifiers and not just give up
                    log::error!("Failed to load mod identifiers: {}", e);
                }
            }
        });

        let mods_dir_path = self.fml_config.mods_dir_path.clone();
        let manage_mod_list = self.manage_mod_list.clone();
        tokio::spawn(async move {
            let installed_mods = match installed_mods::read_installed_mods(&mods_dir_path) {
                Ok(mods) => mods,
                Err(e) => {
                    log::error!("Error reading installed mods: {}", e);
                    vec![]
                }
            };

            let mod_list = mod_list::ModList::load_or_create(&mods_dir_path).unwrap();
            manage_mod_list
                .lock()
                .unwrap()
                .set_items(installed_mods, mod_list);
        });
    }

    pub fn quit_gracefully(&mut self) {
        self.should_quit = true;
    }

    pub fn block_style(&self, block: ActiveBlock) -> Style {
        if self.active_block() == block {
            default_active_block_style()
        } else {
            default_block_style()
        }
    }

    pub fn current_tab(&self) -> Tab {
        self.navigation_history.last().unwrap_or(&DEFAULT_ROUTE).tab
    }

    pub fn active_block(&self) -> ActiveBlock {
        self.navigation_history
            .last()
            .unwrap_or(&DEFAULT_ROUTE)
            .active_block
    }

    fn navigate(&mut self, route: Route) {
        self.navigation_history.push(route);
    }

    pub fn navigate_tab(&mut self, tab: Tab) {
        let active_block = match tab {
            Tab::Manage => ActiveBlock::ManageModList,
            Tab::Install => ActiveBlock::InstallModList,
        };
        self.navigate(Route { tab, active_block });
    }

    pub fn navigate_block(&mut self, active_block: ActiveBlock) {
        let tab = match active_block {
            ActiveBlock::ManageModList => Tab::Manage,
            ActiveBlock::InstallModList
            | ActiveBlock::InstallSearch
            | ActiveBlock::InstallModDetails => Tab::Install,
            ActiveBlock::QuitPopup => self.current_tab(),
        };

        self.navigate(Route { tab, active_block });
    }

    pub fn undo_navigation(&mut self) {
        if self.navigation_history.len() > 1 {
            self.navigation_history.pop();
        }
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    pub fn scroll_down(&mut self) {
        self.scroll_offset += 1;
    }

    pub fn save(&self) {
        self.manage_mod_list
            .lock()
            .unwrap()
            .generate_mod_list()
            .save(&self.fml_config.mods_dir_path)
            .unwrap();
    }

    pub fn delete_mod(&self, mod_name: &str) {
        installed_mods::delete_mod(mod_name, &self.fml_config.mods_dir_path).unwrap();

        self.manage_mod_list.lock().unwrap().remove_mod(mod_name);
        self.install_mod_list.lock().unwrap().disable_mod(mod_name);
    }

    pub fn install_mod(&self, mod_: Arc<Mutex<InstallModItem>>) {
        let mod_ = mod_.lock().unwrap().clone();

        self.mod_downloader
            .tx
            .send(ModDownloadRequest {
                mod_name: mod_.mod_identifier.name.clone(),
                ver_req: semver::VersionReq::parse("*").unwrap(),
                username: self.server_settings.username.clone(),
                token: self.server_settings.token.clone(),
                mod_dir: self.fml_config.mods_dir_path.clone(),
            })
            .unwrap();
    }
}

fn default_active_block_style() -> Style {
    Style::default().fg(Color::Yellow)
}

fn default_block_style() -> Style {
    Style::reset()
}
