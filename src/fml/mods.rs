use super::widgets::mod_list::{ListState, ModListItem};

#[derive(Debug, Default)]
pub struct StatefulModList {
    pub state: ListState,
    items: Vec<ModListItem>,
}

impl StatefulModList {
    pub fn with_items(items: Vec<ModListItem>) -> StatefulModList {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        StatefulModList {
            state: list_state,
            items,
        }
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

    pub fn toggle_install(&mut self, index: Option<usize>) {
        let index = match index {
            Some(index) => index,
            None => match self.state.selected() {
                Some(index) => index,
                None => return,
            },
        };
        self.items[index].installed = !self.items[index].installed;
    }

    pub fn items(&mut self, filter: &String) -> Vec<ModListItem> {
        self.items
            .iter()
            .filter(|item| item.factorio_mod.name.contains(filter))
            .cloned()
            .collect()
    }
}
