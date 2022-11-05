use crate::fml::app::{ActiveBlock, Tab, FML};
use crate::fml::event::{Event, KeyCode};

use super::{install_mod_list, install_search, manage_mod_list, quit_popup, install_mod_details};

pub fn handle(event: Event<KeyCode>, app: &mut FML) {
    match event {
        Event::Input(ref key) => match key {
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
                ActiveBlock::InstallModList => {
                    install_mod_list::handle(event, app)
                }
                ActiveBlock::InstallSearch => {
                    install_search::handle(event, app)
                }
                ActiveBlock::ManageModList => {
                    manage_mod_list::handle(event, app)
                }
                ActiveBlock::QuitPopup => {
                    quit_popup::handle(event, app)
                }
                ActiveBlock::InstallModDetails => {
                    install_mod_details::handle(event, app)
                },
            },
        },
        Event::Tick => app.ticks += 1,
    }
}
