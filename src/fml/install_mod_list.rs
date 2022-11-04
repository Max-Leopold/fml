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
    items: Option<Vec<Arc<Mutex<ModItem>>>>,
    filtered_items: Option<Vec<Arc<Mutex<ModItem>>>>,
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
        self.items = Some(
            items
                .into_iter()
                .map(|item| Arc::new(Mutex::new(item)))
                .collect(),
        );
        self.filtered_items = self.items.clone();
    }

    pub fn is_ready(&self) -> bool {
        self.items.is_some()
    }

    pub fn reset_selected(&mut self) {
        self.state.select(None);
    }

    pub fn next(&mut self) {
        if let None = self.filtered_items {
            return;
        }

        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.filtered_items.as_ref().unwrap().len() - 1 {
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
        if let None = self.filtered_items {
            return;
        }

        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_items.as_ref().unwrap().len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn selected_mod(&self) -> Option<Arc<Mutex<ModItem>>> {
        if let None = self.filtered_items {
            return None;
        }

        match self.state.selected() {
            Some(index) => {
                let mod_item = self.filtered_items.as_ref().unwrap().get(index).unwrap();
                Some(mod_item.clone())
            }
            None => None,
        }
    }

    pub fn items(&mut self, filter: &String) -> Vec<EnabledListItem> {
        if let None = self.items {
            return vec![];
        }

        self.filtered_items = Some(
            self.items
                .as_ref()
                .unwrap()
                .iter()
                .filter(|item| {
                    let mod_ = &item.lock().unwrap().mod_;
                    mod_.name.to_lowercase().contains(&filter.to_lowercase())
                        || mod_.title.to_lowercase().contains(&filter.to_lowercase())
                })
                .map(|item| item.clone())
                .collect(),
        );

        self.filtered_items
            .as_ref()
            .unwrap()
            .iter()
            .map(|item| {
                let mod_item = &item.lock().unwrap();
                EnabledListItem::new(mod_item.mod_.title.clone())
                    .enabled(mod_item.download_info.downloaded)
            })
            .collect()
    }
}
