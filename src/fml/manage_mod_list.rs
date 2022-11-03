use std::sync::{Arc, Mutex};

use super::widgets::mod_list::{ListState, ModListItem};

#[derive(Debug, Default)]
pub struct ManageModList {
    pub state: ListState,
    items: Option<Vec<Arc<Mutex<ModListItem>>>>,
    filtered_items: Option<Vec<Arc<Mutex<ModListItem>>>>,
}

impl ManageModList {
    pub fn set_items(&mut self, items: Vec<ModListItem>) {
        self.items = Some(
            items
                .into_iter()
                .map(|item| Arc::new(Mutex::new(item)))
                .collect(),
        );
        self.filtered_items = self.items.clone();
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

    pub fn items(&self) -> Vec<ModListItem> {
        if let None = self.items {
            return vec![];
        }

        self.items
            .as_ref()
            .unwrap()
            .iter()
            .map(|item| item.lock().unwrap().clone())
            .collect()
    }
}
