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
        KeyCode::Left => {
            app.navigate_block(ActiveBlock::InstallModList);
        }
        KeyCode::Up => {
            app.scroll_up();
        }
        KeyCode::Down => {
            app.scroll_down();
        }
        _ => {}
    }
}
