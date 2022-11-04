use crate::factorio::api;
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
            KeyCode::Up => {
                app.install_mod_list.lock().unwrap().previous();
            }
            KeyCode::Down => {
                app.install_mod_list.lock().unwrap().next();
            }
            KeyCode::Enter => {
                let mod_ = app.install_mod_list.lock().unwrap().selected_mod();
                if let Some(mod_) = mod_ {
                    let factorio_mod = &mod_.lock().unwrap().mod_.clone();
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
                                mod_.lock().unwrap().download_info.download_perc = x;
                            }),
                        )
                        .await
                        .unwrap();

                        mod_.lock().unwrap().download_info.downloaded = true;
                    });
                }
            }
            _ => {}
        },
        _ => {}
    }
}
