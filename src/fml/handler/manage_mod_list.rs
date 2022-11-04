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
            KeyCode::Enter => match app.manage_mod_list.lock().unwrap().selected_mod() {
                Some(mod_) => {
                    let mut mod_ = mod_.lock().unwrap();
                    mod_.enabled = !mod_.enabled;
                }
                None => {}
            },
            KeyCode::Char('d') => {
                let mod_ = app.manage_mod_list.lock().unwrap().selected_mod();
                if let Some(mod_) = mod_ {
                    let mod_name = mod_.lock().unwrap().mod_.name.clone();
                    app.delete_mod(&mod_name);
                }
            },
            _ => {}
        },
        _ => {}
    }
}
