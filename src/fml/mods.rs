use super::widgets::mod_list::{ListState, ModListItem};
use std::sync::{Arc, Mutex};

#[derive(Debug, Default)]
pub struct StatefulModList {
    pub state: ListState,
    items: Vec<Arc<Mutex<ModListItem>>>,
    filtered_items: Vec<Arc<Mutex<ModListItem>>>,
}

impl StatefulModList {
    pub fn set_items(&mut self, items: Vec<ModListItem>) {
        self.items = items
            .into_iter()
            .map(|item| Arc::new(Mutex::new(item)))
            .collect();
        self.filtered_items = self.items.clone();
    }

    pub fn reset_selected(&mut self) {
        self.state.select(None);
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
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
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn toggle_install(&mut self, index: Option<usize>) -> Option<bool> {
        let index = match index {
            Some(index) => index,
            None => match self.state.selected() {
                Some(index) => index,
                None => return None,
            },
        };
        // Toggle installed value for the selected mod
        let mod_item = self.filtered_items.get(index).unwrap();
        let mut mod_item = mod_item.lock().unwrap();
        mod_item.installed = !mod_item.installed;
        Some(mod_item.installed)
    }

    pub fn selected_mod(&self) -> Option<ModListItem> {
        match self.state.selected() {
            Some(index) => {
                let mod_item = self.filtered_items.get(index).unwrap();
                let mod_item = mod_item.lock().unwrap();
                Some(mod_item.clone())
            }
            None => None,
        }
    }

    pub fn items(&mut self, filter: &String) -> Vec<ModListItem> {
        self.filtered_items = self
            .items
            .iter()
            .filter(|item| {
                item.lock()
                    .unwrap()
                    .factorio_mod
                    .name
                    .to_lowercase()
                    .contains(&filter.to_lowercase())
            })
            .map(|item| item.clone())
            .collect();

        self.filtered_items
            .iter()
            .map(|item| item.lock().unwrap().clone())
            .collect()
    }
}
