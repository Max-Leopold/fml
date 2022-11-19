use crate::fml::app::FML;
use crate::fml::event::KeyCode;

pub fn handle(key: KeyCode, app: &mut FML) {
    match key {
        KeyCode::Esc => {
            app.undo_navigation();
        }
        KeyCode::Char('y') => {
            app.save();
            app.quit_gracefully();
        }
        KeyCode::Char('n') => {
            app.quit_gracefully();
        }
        _ => {}
    }
}
