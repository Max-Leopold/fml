use super::widgets::mod_list::{ListState, ModListItem};
use std::sync::{Arc, Mutex};

#[derive(Debug, Default)]
pub struct InstallModList {
    pub state: ListState,
    items: Option<Vec<Arc<Mutex<ModListItem>>>>,
    filtered_items: Option<Vec<Arc<Mutex<ModListItem>>>>,
}

impl InstallModList {
    pub fn set_items(&mut self, items: Vec<ModListItem>) {
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

    pub fn selected_mod(&self) -> Option<Arc<Mutex<ModListItem>>> {
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

    pub fn items(&mut self, filter: &String) -> Vec<ModListItem> {
        if let None = self.items {
            return vec![];
        }

        self.filtered_items = Some(
            self.items
                .as_ref()
                .unwrap()
                .iter()
                .filter(|item| {
                    let mod_ = &item.lock().unwrap().mod_item.mod_;
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
            .map(|item| item.lock().unwrap().clone())
            .collect()
    }
}
