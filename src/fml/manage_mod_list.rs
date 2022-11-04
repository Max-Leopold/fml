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
    items: Option<Vec<Arc<Mutex<ManageModItem>>>>,
    filtered_items: Option<Vec<Arc<Mutex<ManageModItem>>>>,
}

impl ManageModList {
    pub fn set_items(&mut self, items: Vec<InstalledMod>, mod_list: ModList) {
        self.items = Some(
            items
                .into_iter()
                .map(|item| {
                    let item_name = item.name.clone();
                    Arc::new(Mutex::new(ManageModItem {
                        mod_: item,
                        enabled: mod_list.mods.contains_key(&item_name)
                            && mod_list.mods.get(&item_name).unwrap().enabled,
                    }))
                })
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

    pub fn selected_mod(&self) -> Option<Arc<Mutex<ManageModItem>>> {
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

    pub fn add_mod(&mut self, mod_: InstalledMod, enabled: bool) {
        if let None = self.items {
            self.items = Some(Vec::new());
        }

        if let None = self.filtered_items {
            self.filtered_items = Some(Vec::new());
        }

        let mod_item = Arc::new(Mutex::new(ManageModItem { mod_, enabled }));

        self.items.as_mut().unwrap().push(mod_item.clone());
        self.filtered_items.as_mut().unwrap().push(mod_item.clone());
    }

    pub fn remove_mod(&mut self, mod_name: &str) {
        if let None = self.items {
            return;
        }

        if let None = self.filtered_items {
            return;
        }

        let index = self
            .items
            .as_ref()
            .unwrap()
            .iter()
            .position(|mod_item| mod_item.lock().unwrap().mod_.name == mod_name);

        if let Some(index) = index {
            self.items.as_mut().unwrap().remove(index);
        }

        let index = self
            .filtered_items
            .as_ref()
            .unwrap()
            .iter()
            .position(|mod_item| mod_item.lock().unwrap().mod_.name == mod_name);

        if let Some(index) = index {
          self.filtered_items.as_mut().unwrap().remove(index);
        }
    }

    pub fn items(&self) -> Vec<EnabledListItem> {
        if let None = self.items {
            return vec![];
        }

        self.items
            .as_ref()
            .unwrap()
            .iter()
            .map(|item| {
                let item = item.lock().unwrap();
                EnabledListItem::new(item.mod_.title.clone()).enabled(item.enabled)
            })
            .collect()
    }

    pub fn generate_mod_list(&self) -> ModList {
        let mut mod_list = ModList::new();

        if let None = self.items {
            return mod_list;
        }

        for item in self.items.as_ref().unwrap() {
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
