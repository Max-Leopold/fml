use crate::factorio::api;
use crate::fml::app::FML;
use crate::fml::event::{Event, KeyCode};

use super::handler::EventHandler;

pub struct InstallModListHandler {}

impl EventHandler for InstallModListHandler {
    fn handle(event: Event<KeyCode>, app: &mut FML) {
        match event {
            Event::Input(ref input) => match input {
                KeyCode::Char('/') => {
                    app.active_block = crate::fml::app::ActiveBlock::InstallSearch;
                }
                KeyCode::Backspace | KeyCode::Char(_) => {
                    app.active_block = crate::fml::app::ActiveBlock::InstallSearch;
                    app.events.tx.send(event).unwrap();
                }
                KeyCode::Up => {
                    app.stateful_mod_list.lock().unwrap().previous();
                }
                KeyCode::Down => {
                    app.stateful_mod_list.lock().unwrap().next();
                }
                KeyCode::Enter => {
                    let mod_ = app.stateful_mod_list.lock().unwrap().selected_mod();
                    if let Some(mod_) = mod_ {
                        let factorio_mod = &mod_.lock().unwrap().mod_item.mod_.clone();
                        let token = app.server_settings.token.clone();
                        let username = app.server_settings.username.clone();
                        let mod_name = factorio_mod.name.clone();
                        let mod_dir = app.fml_config.mods_dir_path.clone();
                        tokio::spawn(async move {
                            api::download_mod(
                                &mod_name,
                                &username,
                                &token,
                                &mod_dir,
                                Some(|x| {
                                    mod_.lock().unwrap().mod_item.download_info.download_perc = x;
                                }),
                            )
                            .await
                            .unwrap();

                            mod_.lock().unwrap().mod_item.download_info.downloaded = true;
                        });
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
}
