use tui::widgets::ListState;

use crate::factorio::api::Mod;

#[derive(Debug, Default)]
pub struct ModList {
    pub state: ListState,
    pub items: Vec<Mod>,
}

impl ModList {
  pub fn with_items(items: Vec<Mod>) -> ModList {
    let mut list_state = ListState::default();
    list_state.select(Some(0));

    ModList {
      state: list_state,
      items
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
      None => 0
    };
    self.state.select(Some(i));
  }
}
