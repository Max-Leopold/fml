use crate::fml::app::{ActiveBlock, Tab, FML};
use crate::fml::event::KeyCode;

use super::{install_mod_details, install_mod_list, install_search, manage_mod_list, quit_popup};

pub fn handle(key: KeyCode, app: &mut FML) {
    match key {
        KeyCode::Ctrl('c') => {
            app.navigate_block(ActiveBlock::QuitPopup);
        }
        KeyCode::Tab => match app.current_tab() {
            Tab::Manage => {
                app.navigate_tab(Tab::Install);
            }
            Tab::Install => {
                app.navigate_tab(Tab::Manage);
            }
        },
        _ => match app.active_block() {
            ActiveBlock::InstallModList => install_mod_list::handle(key, app),
            ActiveBlock::InstallSearch => install_search::handle(key, app),
            ActiveBlock::ManageModList => manage_mod_list::handle(key, app),
            ActiveBlock::QuitPopup => quit_popup::handle(key, app),
            ActiveBlock::InstallModDetails => install_mod_details::handle(key, app),
        },
    }
}
