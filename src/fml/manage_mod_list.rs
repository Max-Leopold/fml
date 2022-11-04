use std::sync::{Arc, Mutex};

use super::widgets::enabled_list::{EnabledListItem, ListState};
use crate::factorio::installed_mods::InstalledMod;
use crate::factorio::mod_list::{ModEntry, ModList};

#[derive(Debug, Default)]
pub struct ManageModItem {
    pub mod_: InstalledMod,
    pub enabled: bool,
}

#[derive(Debug, Default)]
pub struct ManageModList {
    pub state: ListState,
    items: Vec<Arc<Mutex<ManageModItem>>>,
    filter: String,
}

impl ManageModList {
    pub fn set_items(&mut self, items: Vec<InstalledMod>, mod_list: ModList) {
        self.items = items
            .into_iter()
            .map(|item| {
                let item_name = item.name.clone();
                Arc::new(Mutex::new(ManageModItem {
                    mod_: item,
                    enabled: mod_list.mods.contains_key(&item_name)
                        && mod_list.mods.get(&item_name).unwrap().enabled,
                }))
            })
            .collect()
    }

    pub fn set_filter(&mut self, filter: String) {
        self.filter = filter;
    }

    fn filtered_items(&self) -> Vec<Arc<Mutex<ManageModItem>>> {
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

    pub fn selected_mod(&self) -> Option<Arc<Mutex<ManageModItem>>> {
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

    pub fn add_mod(&mut self, mod_: InstalledMod, enabled: bool) {
        let mod_item = Arc::new(Mutex::new(ManageModItem { mod_, enabled }));

        self.items.push(mod_item.clone());
    }

    pub fn remove_mod(&mut self, mod_name: &str) {
        let index = self
            .items
            .iter()
            .position(|mod_item| mod_item.lock().unwrap().mod_.name == mod_name);

        if let Some(index) = index {
            self.items.remove(index);
        }
    }

    pub fn items(&self) -> Vec<EnabledListItem> {
        self.items
            .iter()
            .map(|item| {
                let item = item.lock().unwrap();
                EnabledListItem::new(item.mod_.title.clone()).enabled(item.enabled)
            })
            .collect()
    }

    pub fn generate_mod_list(&self) -> ModList {
        let mut mod_list = ModList::new();

        for item in &self.items {
            let item = item.lock().unwrap();
            mod_list.mods.insert(
                item.mod_.name.clone(),
                ModEntry {
                    name: item.mod_.name.clone(),
                    enabled: item.enabled,
                    version: None,
                },
            );
        }

        mod_list
    }
}
