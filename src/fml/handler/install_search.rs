use crate::fml::app::{ActiveBlock, FML};
use crate::fml::event::{Event, KeyCode};

pub fn handle(key: KeyCode, app: &mut FML) {
    match key {
        KeyCode::Char(c) => {
            app.install_mod_list.lock().unwrap().reset_selected();
            app.install_mod_list.lock().unwrap().filter.push(c);
        }
        KeyCode::Backspace => {
            app.install_mod_list.lock().unwrap().reset_selected();
            app.install_mod_list.lock().unwrap().filter.pop();
        }
        KeyCode::Enter => {
            app.navigate_block(ActiveBlock::InstallModList);
        }
        KeyCode::Down | KeyCode::Up => {
            app.navigate_block(ActiveBlock::InstallModList);
            app.events.tx.send(Event::Input(key)).unwrap();
        }
        _ => {}
    }
}
