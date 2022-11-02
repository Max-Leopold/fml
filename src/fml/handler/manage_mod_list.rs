use crate::fml::app::FML;
use crate::fml::event::{Event, KeyCode};

pub fn handle(event: Event<KeyCode>, app: &mut FML) {
    match event {
        Event::Input(ref input) => match input {
            KeyCode::Up => {
                app.manage_mod_list.lock().unwrap().previous();
            }
            KeyCode::Down => {
                app.manage_mod_list.lock().unwrap().next();
            }
            _ => {}
        },
        _ => {}
    }
}
