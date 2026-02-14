use std::time::Instant;

use crate::factorio::installed::InstalledMod;
use crate::factorio::types::{ModListEntry, ServerSettings};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Manage,
    Install,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveBlock {
    ManageModList,
    InstallModList,
    InstallSearch,
    QuitPopup,
}

#[derive(Debug, Clone)]
pub struct ManageMod {
    pub installed_mod: InstalledMod,
    pub enabled: bool,
    pub pending: bool,
}

pub struct App {
    pub tab: Tab,
    pub active_block: ActiveBlock,
    pub install_mods: Vec<ModListEntry>,
    pub install_filter: String,
    pub install_selected: Option<usize>,
    pub manage_mods: Vec<ManageMod>,
    pub manage_selected: Option<usize>,
    pub status_message: Option<(String, Instant)>,
    pub factorio_version: String,
    pub server_settings: ServerSettings,
    pub mods_dir: String,
    pub should_quit: bool,
    pub show_quit_popup: bool,
    pub loading: bool,
    pub installing: bool,
}

impl App {
    pub fn new(factorio_version: String, server_settings: ServerSettings, mods_dir: String) -> Self {
        App {
            tab: Tab::Manage,
            active_block: ActiveBlock::ManageModList,
            install_mods: Vec::new(),
            install_filter: String::new(),
            install_selected: None,
            manage_mods: Vec::new(),
            manage_selected: None,
            status_message: None,
            factorio_version,
            server_settings,
            mods_dir,
            should_quit: false,
            show_quit_popup: false,
            loading: true,
            installing: false,
        }
    }

    pub fn filtered_install_mods(&self) -> Vec<&ModListEntry> {
        if self.install_filter.is_empty() {
            return self.install_mods.iter().collect();
        }
        let filter = self.install_filter.to_lowercase();
        self.install_mods
            .iter()
            .filter(|m| {
                m.name.to_lowercase().contains(&filter)
                    || m.title.to_lowercase().contains(&filter)
            })
            .collect()
    }

    pub fn set_status(&mut self, msg: String) {
        self.status_message = Some((msg, Instant::now()));
    }

    pub fn clear_expired_status(&mut self) {
        if let Some((_, time)) = &self.status_message {
            if time.elapsed().as_secs() >= 5 {
                self.status_message = None;
            }
        }
    }

    pub fn is_installed(&self, mod_name: &str) -> bool {
        self.manage_mods
            .iter()
            .any(|m| m.installed_mod.name == mod_name)
    }

    /// Returns manage_mods indices in visual order: saved mods first, then pending.
    pub fn manage_display_order(&self) -> Vec<usize> {
        let mut order: Vec<usize> = Vec::new();
        for (i, m) in self.manage_mods.iter().enumerate() {
            if !m.pending {
                order.push(i);
            }
        }
        for (i, m) in self.manage_mods.iter().enumerate() {
            if m.pending {
                order.push(i);
            }
        }
        order
    }

    pub fn move_up(&mut self) {
        match self.active_block {
            ActiveBlock::ManageModList => {
                let order = self.manage_display_order();
                if order.is_empty() {
                    return;
                }
                if let Some(sel) = self.manage_selected {
                    if let Some(pos) = order.iter().position(|&idx| idx == sel) {
                        if pos > 0 {
                            self.manage_selected = Some(order[pos - 1]);
                        }
                    }
                }
            }
            ActiveBlock::InstallModList => {
                if let Some(sel) = self.install_selected {
                    if sel > 0 {
                        self.install_selected = Some(sel - 1);
                    }
                }
            }
            _ => {}
        }
    }

    pub fn move_down(&mut self) {
        match self.active_block {
            ActiveBlock::ManageModList => {
                let order = self.manage_display_order();
                if order.is_empty() {
                    return;
                }
                if let Some(sel) = self.manage_selected {
                    if let Some(pos) = order.iter().position(|&idx| idx == sel) {
                        if pos < order.len() - 1 {
                            self.manage_selected = Some(order[pos + 1]);
                        }
                    }
                } else {
                    self.manage_selected = Some(order[0]);
                }
            }
            ActiveBlock::InstallModList => {
                let len = self.filtered_install_mods().len();
                if len > 0 {
                    let sel = self.install_selected.unwrap_or(0);
                    if sel < len - 1 {
                        self.install_selected = Some(sel + 1);
                    }
                }
            }
            _ => {}
        }
    }

    pub fn select_tab(&mut self, tab: Tab) {
        self.tab = tab;
        match tab {
            Tab::Manage => self.active_block = ActiveBlock::ManageModList,
            Tab::Install => self.active_block = ActiveBlock::InstallModList,
        }
    }
}
