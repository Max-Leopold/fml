use crate::fml::app::{ActiveBlock, FML};
use crate::fml::event::{Event, KeyCode};

pub fn handle(event: Event<KeyCode>, app: &mut FML) {
    match event {
        Event::Input(ref input) => match input {
            KeyCode::Char('/') => {
                app.navigate_block(ActiveBlock::InstallSearch);
            }
            KeyCode::Backspace | KeyCode::Char(_) => {
                app.navigate_block(ActiveBlock::InstallSearch);
                app.events.tx.send(event).unwrap();
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
        },
        _ => {}
    }
}
