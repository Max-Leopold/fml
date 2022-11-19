use crate::fml::app::{ActiveBlock, FML};
use crate::fml::event::{Event, KeyCode};

pub fn handle(key: KeyCode, app: &mut FML) {
    match key {
        KeyCode::Char('/') => {
            app.navigate_block(ActiveBlock::InstallSearch);
        }
        KeyCode::Backspace | KeyCode::Char(_) => {
            app.navigate_block(ActiveBlock::InstallSearch);
            app.events.tx.send(Event::Input(key)).unwrap();
        }
        KeyCode::Up => {
            app.scroll_offset = 0;
            app.install_mod_list.lock().unwrap().previous();
        }
        KeyCode::Down => {
            app.scroll_offset = 0;
            app.install_mod_list.lock().unwrap().next();
        }
        KeyCode::Right => {
            app.navigate_block(ActiveBlock::InstallModDetails);
        }
        KeyCode::Enter => {
            let mod_ = app.install_mod_list.lock().unwrap().selected_mod();
            if let Some(mod_) = mod_ {
                app.install_mod(mod_);
            }
        }
        _ => {}
    }
}
