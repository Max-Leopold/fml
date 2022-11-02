use crate::fml::app::FML;
use crate::fml::event::{Event, KeyCode};

pub fn handle(event: Event<KeyCode>, app: &mut FML) {
    match event {
        Event::Input(ref key) => match key {
            KeyCode::Esc => {
                app.undo_navigation();
            }
            KeyCode::Char('y') => {
                app.mod_list.save().unwrap();
                app.quit_gracefully();
            }
            KeyCode::Char('n') => {
                app.quit_gracefully();
            }
            _ => {}
        },
        Event::Tick => {}
    }
}
