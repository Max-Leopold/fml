use crate::factorio::api;

use super::widgets::enabled_list::{EnabledListItem, ListState};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct InstallModItem {
    pub mod_identifier: api::ModIdentifier,
    pub download_info: DownloadInfo,
    pub loading: bool,
}

#[derive(Debug, Clone, Default)]
pub struct DownloadInfo {
    pub downloaded: bool,
    pub downloading: bool,
    pub download_perc: u16,
    pub versions: Vec<String>,
}

#[derive(Debug, Default)]
pub struct InstallModList {
    pub state: ListState,
    pub filter: String,
    items: Vec<Arc<Mutex<InstallModItem>>>,
}

impl InstallModItem {
    pub fn new(mod_identifier: api::ModIdentifier) -> InstallModItem {
        InstallModItem {
            mod_identifier,
            loading: false,
            download_info: DownloadInfo::default(),
        }
    }
}

impl InstallModList {
    pub fn set_items(&mut self, items: Vec<InstallModItem>) {
        self.items = items
            .into_iter()
            .map(|item| Arc::new(Mutex::new(item)))
            .collect();
    }

    pub fn is_ready(&self) -> bool {
        !self.items.is_empty()
    }

    pub fn reset_selected(&mut self) {
        self.state.select(None);
    }

    fn filtered_items(&self) -> Vec<Arc<Mutex<InstallModItem>>> {
        self.items
            .iter()
            .filter(|mod_item| {
                let mod_identifier = &mod_item.lock().unwrap().mod_identifier;
                mod_identifier
                    .name
                    .to_lowercase()
                    .contains(&self.filter.to_lowercase())
                    || mod_identifier
                        .title
                        .to_lowercase()
                        .contains(&self.filter.to_lowercase())
            })
            .map(|mod_| mod_.clone())
            .collect()
    }

    pub fn next(&mut self) {
        if self.filtered_items().is_empty() {
            return;
        }

        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.filtered_items().len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };

        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.filtered_items().is_empty() {
            return;
        }

        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_items().len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn selected_mod(&self) -> Option<Arc<Mutex<InstallModItem>>> {
        if self.filtered_items().is_empty() {
            return None;
        }

        match self.state.selected() {
            Some(index) => {
                let filtered_items = self.filtered_items();
                let mod_item = filtered_items.get(index).unwrap();
                Some(mod_item.clone())
            }
            None => None,
        }
    }

    pub fn disable_mod(&mut self, mod_name: &str) {
        let mod_ = self
            .items
            .iter()
            .find(|mod_item| mod_item.lock().unwrap().mod_identifier.name == mod_name);

        if let Some(mod_) = mod_ {
            mod_.lock().unwrap().download_info.downloaded = false;
        }
    }

    pub fn enable_mod(&mut self, mod_name: &str) {
        let mod_ = self
            .items
            .iter()
            .find(|mod_item| mod_item.lock().unwrap().mod_identifier.name == mod_name);

        if let Some(mod_) = mod_ {
            mod_.lock().unwrap().download_info.downloaded = true;
        }
    }

    pub fn items(&self) -> Vec<EnabledListItem> {
        if self.items.is_empty() {
            return vec![];
        }

        self.filtered_items()
            .iter()
            .map(|item| {
                let mod_item = &item.lock().unwrap();
                EnabledListItem::new(mod_item.mod_identifier.title.clone())
                    .enabled(mod_item.download_info.downloaded)
            })
            .collect()
    }
}
