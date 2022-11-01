use crate::fml::app::{ActiveBlock, Tabs, FML};
use crate::fml::event::{Event, KeyCode};

use super::install_mod_list::InstallModListHandler;
use super::install_search::InstallSearchHandler;
use super::manage_mod_list::ManageModListHandler;

pub trait EventHandler {
    fn handle(event: Event<KeyCode>, app: &mut FML);
}

pub struct Handler {}

impl EventHandler for Handler {
    fn handle(event: Event<KeyCode>, app: &mut FML) {
        match event {
            Event::Input(ref key) => match key {
                KeyCode::Ctrl('c') => {
                    app.quit();
                }
                KeyCode::Tab => match app.current_tab {
                    Tabs::Manage => {
                      app.current_tab = Tabs::Install;
                      app.active_block = ActiveBlock::InstallModList;
                    },
                    Tabs::Install => {
                      app.current_tab = Tabs::Manage;
                      app.active_block = ActiveBlock::ManageModList;
                    },
                },
                _ => match app.active_block {
                    ActiveBlock::InstallModList => {
                        InstallModListHandler::handle(event, app);
                    }
                    ActiveBlock::InstallSearch => {
                        InstallSearchHandler::handle(event, app);
                    }
                    ActiveBlock::ManageModList => {
                        ManageModListHandler::handle(event, app);
                    }
                },
            },
            Event::Tick => app.ticks += 1,
        }
    }
}
