use crate::fml::app::FML;
use crate::fml::event::{Event, KeyCode};

use super::handler::EventHandler;

pub struct InstallSearchHandler {}

impl EventHandler for InstallSearchHandler {
    fn handle(event: Event<KeyCode>, app: &mut FML) {
        match event {
            Event::Input(input) => match input {
                KeyCode::Char(c) => {
                    app.stateful_mod_list.lock().unwrap().reset_selected();
                    app.filter.push(c);
                }
                KeyCode::Backspace => {
                    app.stateful_mod_list.lock().unwrap().reset_selected();
                    app.filter.pop();
                }
                KeyCode::Esc => {
                    app.active_block = crate::fml::app::ActiveBlock::InstallModList;
                }
                _ => {}
            },
            _ => {}
        }
    }
}
