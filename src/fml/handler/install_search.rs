use crate::fml::app::{ActiveBlock, FML};
use crate::fml::event::{Event, KeyCode};

pub fn handle(event: Event<KeyCode>, app: &mut FML) {
    match event {
        Event::Input(ref input) => match input {
            KeyCode::Char(c) => {
                app.stateful_mod_list.lock().unwrap().reset_selected();
                app.filter.push(*c);
            }
            KeyCode::Backspace => {
                app.stateful_mod_list.lock().unwrap().reset_selected();
                app.filter.pop();
            }
            KeyCode::Enter => {
                app.navigate_block(ActiveBlock::InstallModList);
            }
            KeyCode::Down | KeyCode::Up => {
                app.navigate_block(ActiveBlock::InstallModList);
                app.events.tx.send(event).unwrap();
            }
            _ => {}
        },
        _ => {}
    }
}
