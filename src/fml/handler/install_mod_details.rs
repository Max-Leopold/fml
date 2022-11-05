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
        },
        _ => {}
    }
}
