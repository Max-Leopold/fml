use crate::factorio::api;

use super::widgets::enabled_list::{EnabledListItem, ListState};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct ModItem {
    pub mod_: api::Mod,
    pub loading: bool,
    pub download_info: DownloadInfo,
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
    items: Vec<Arc<Mutex<ModItem>>>,
}

impl ModItem {
    pub fn new(mod_: api::Mod) -> ModItem {
        ModItem {
            mod_,
            loading: false,
            download_info: DownloadInfo::default(),
        }
    }
}

impl InstallModList {
    pub fn set_items(&mut self, items: Vec<ModItem>) {
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

    fn filtered_items(&self) -> Vec<Arc<Mutex<ModItem>>> {
        self.items
            .iter()
            .filter(|mod_item| {
                let mod_ = &mod_item.lock().unwrap().mod_;
                mod_.name.contains(&self.filter) || mod_.title.contains(&self.filter)
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

    pub fn selected_mod(&self) -> Option<Arc<Mutex<ModItem>>> {
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

    pub fn items(&self) -> Vec<EnabledListItem> {
        if self.items.is_empty() {
            return vec![];
        }

        self.filtered_items()
            .iter()
            .map(|item| {
                let mod_item = &item.lock().unwrap();
                EnabledListItem::new(mod_item.mod_.title.clone())
                    .enabled(mod_item.download_info.downloaded)
            })
            .collect()
    }
}
