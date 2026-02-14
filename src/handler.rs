use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;
use tokio::sync::mpsc;

use crate::app::{ActiveBlock, App, ManageMod, Tab};
use crate::event::{AppEvent, InstallResult};
use crate::factorio::{api, installed, mod_list::ModList, resolver};

pub fn handle_event(event: AppEvent, app: &mut App, tx: mpsc::UnboundedSender<AppEvent>) {
    match event {
        AppEvent::Key(key) => handle_key(key, app, tx),
        AppEvent::Tick => app.clear_expired_status(),
        AppEvent::ModListLoaded(result) => match result {
            Ok(mods) => {
                app.install_mods = mods;
                app.loading = false;
                if app.install_mods.is_empty() {
                    app.set_status("No mods found for this Factorio version".to_string());
                } else {
                    app.install_selected = Some(0);
                }
            }
            Err(e) => {
                app.loading = false;
                app.set_status(format!("Failed to load mod list: {}", e));
            }
        },
        AppEvent::ModInstalled(result) => {
            app.installing = false;
            match result {
                Ok(install_result) => {
                    // Build set of previously known mods with their enabled state
                    let prev_state: HashMap<String, bool> = app
                        .manage_mods
                        .iter()
                        .map(|m| (m.installed_mod.name.clone(), m.enabled))
                        .collect();

                    // Refresh manage mods from disk
                    app.manage_mods = install_result
                        .installed_mods
                        .into_iter()
                        .map(|m| {
                            // Preserve existing enabled state, new mods default to enabled
                            let enabled = prev_state.get(&m.name).copied().unwrap_or(true);
                            ManageMod {
                                installed_mod: m,
                                enabled,
                            }
                        })
                        .collect();

                    if install_result.dependency_count > 0 {
                        app.set_status(format!(
                            "Installed {} + {} dependencies",
                            install_result.mod_name, install_result.dependency_count
                        ));
                    } else {
                        app.set_status(format!("Installed {}", install_result.mod_name));
                    }
                }
                Err(e) => {
                    app.set_status(format!("Install failed: {}", e));
                }
            }
        }
        AppEvent::ModDeleted(result) => match result {
            Ok(name) => {
                app.manage_mods.retain(|m| m.installed_mod.name != name);
                // Adjust selection
                if !app.manage_mods.is_empty() {
                    if let Some(sel) = app.manage_selected {
                        if sel >= app.manage_mods.len() {
                            app.manage_selected = Some(app.manage_mods.len() - 1);
                        }
                    }
                } else {
                    app.manage_selected = None;
                }
                app.set_status(format!("Deleted {}", name));
            }
            Err(e) => {
                app.set_status(format!("Delete failed: {}", e));
            }
        },
        AppEvent::InstalledModsLoaded(result) => match result {
            Ok((mods, mod_list)) => {
                app.manage_mods = mods
                    .into_iter()
                    .map(|m| {
                        let enabled = mod_list.is_enabled(&m.name);
                        ManageMod {
                            enabled,
                            installed_mod: m,
                        }
                    })
                    .collect();
                if !app.manage_mods.is_empty() {
                    app.manage_selected = Some(0);
                }
            }
            Err(e) => {
                app.set_status(format!("Failed to read installed mods: {}", e));
            }
        },
        AppEvent::Error(msg) => {
            app.set_status(msg);
        }
    }
}

fn handle_key(key: KeyEvent, app: &mut App, tx: mpsc::UnboundedSender<AppEvent>) {
    // Quit popup takes priority
    if app.show_quit_popup {
        handle_quit_popup(key, app);
        return;
    }

    // Global keys
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        app.show_quit_popup = true;
        app.active_block = ActiveBlock::QuitPopup;
        return;
    }

    if key.code == KeyCode::Tab {
        match app.tab {
            Tab::Manage => app.select_tab(Tab::Install),
            Tab::Install => app.select_tab(Tab::Manage),
        }
        return;
    }

    match app.active_block {
        ActiveBlock::ManageModList => handle_manage_keys(key, app, tx),
        ActiveBlock::InstallModList => handle_install_list_keys(key, app, tx),
        ActiveBlock::InstallSearch => handle_search_keys(key, app),
        ActiveBlock::QuitPopup => handle_quit_popup(key, app),
    }
}

fn handle_manage_keys(key: KeyEvent, app: &mut App, tx: mpsc::UnboundedSender<AppEvent>) {
    match key.code {
        KeyCode::Up => app.move_up(),
        KeyCode::Down => app.move_down(),
        KeyCode::Enter => {
            // Toggle enable/disable
            if let Some(sel) = app.manage_selected {
                if let Some(m) = app.manage_mods.get_mut(sel) {
                    m.enabled = !m.enabled;
                }
            }
        }
        KeyCode::Char('d') => {
            // Delete mod
            if let Some(sel) = app.manage_selected {
                if let Some(m) = app.manage_mods.get(sel) {
                    let name = m.installed_mod.name.clone();
                    let version = m.installed_mod.version.clone();
                    let mods_dir = app.mods_dir.clone();
                    tokio::spawn(async move {
                        let result = installed::delete_mod(&name, &version, &mods_dir);
                        let _ = tx.send(AppEvent::ModDeleted(result.map(|_| name)));
                    });
                }
            }
        }
        _ => {}
    }
}

fn handle_install_list_keys(key: KeyEvent, app: &mut App, tx: mpsc::UnboundedSender<AppEvent>) {
    match key.code {
        KeyCode::Up => app.move_up(),
        KeyCode::Down => app.move_down(),
        KeyCode::Char('/') => {
            app.active_block = ActiveBlock::InstallSearch;
        }
        KeyCode::Enter => {
            if app.installing {
                app.set_status("Installation already in progress...".to_string());
                return;
            }

            let filtered = app.filtered_install_mods();
            if let Some(sel) = app.install_selected {
                if let Some(entry) = filtered.get(sel) {
                    let mod_name = entry.name.clone();

                    if app.is_installed(&mod_name) {
                        app.set_status(format!("{} is already installed", mod_name));
                        return;
                    }

                    app.installing = true;
                    app.set_status(format!("Installing {}...", mod_name));

                    let factorio_version = app.factorio_version.clone();
                    let username = app.server_settings.username.clone();
                    let token = app.server_settings.token.clone();
                    let mods_dir = app.mods_dir.clone();

                    // Build installed mods map for the resolver
                    let installed_map: HashMap<String, semver::Version> = app
                        .manage_mods
                        .iter()
                        .map(|m| {
                            (
                                m.installed_mod.name.clone(),
                                m.installed_mod.version.clone(),
                            )
                        })
                        .collect();

                    let tx_clone = tx.clone();
                    tokio::spawn(async move {
                        let result = do_install(
                            &mod_name,
                            &factorio_version,
                            &username,
                            &token,
                            &mods_dir,
                            &installed_map,
                        )
                        .await;
                        let _ = tx_clone.send(AppEvent::ModInstalled(result));
                    });
                }
            }
        }
        KeyCode::Char(c) => {
            // Start typing in search
            app.active_block = ActiveBlock::InstallSearch;
            app.install_filter.push(c);
            app.install_selected = if app.filtered_install_mods().is_empty() {
                None
            } else {
                Some(0)
            };
        }
        _ => {}
    }
}

fn handle_search_keys(key: KeyEvent, app: &mut App) {
    match key.code {
        KeyCode::Esc => {
            app.active_block = ActiveBlock::InstallModList;
        }
        KeyCode::Enter | KeyCode::Down => {
            app.active_block = ActiveBlock::InstallModList;
            if !app.filtered_install_mods().is_empty() && app.install_selected.is_none() {
                app.install_selected = Some(0);
            }
        }
        KeyCode::Char(c) => {
            app.install_filter.push(c);
            app.install_selected = if app.filtered_install_mods().is_empty() {
                None
            } else {
                Some(0)
            };
        }
        KeyCode::Backspace => {
            app.install_filter.pop();
            app.install_selected = if app.filtered_install_mods().is_empty() {
                None
            } else {
                Some(0)
            };
        }
        _ => {}
    }
}

fn handle_quit_popup(key: KeyEvent, app: &mut App) {
    match key.code {
        KeyCode::Char('y') => {
            // Save mod-list.json and quit
            let mut mod_list = ModList::load_or_create(&app.mods_dir).unwrap_or_else(|_| ModList::new());
            for m in &app.manage_mods {
                mod_list.set_enabled(&m.installed_mod.name, m.enabled);
            }
            if let Err(e) = mod_list.save(&app.mods_dir) {
                app.set_status(format!("Failed to save mod-list.json: {}", e));
                app.show_quit_popup = false;
                app.active_block = match app.tab {
                    Tab::Manage => ActiveBlock::ManageModList,
                    Tab::Install => ActiveBlock::InstallModList,
                };
                return;
            }
            app.should_quit = true;
        }
        KeyCode::Char('n') => {
            app.should_quit = true;
        }
        KeyCode::Esc => {
            app.show_quit_popup = false;
            app.active_block = match app.tab {
                Tab::Manage => ActiveBlock::ManageModList,
                Tab::Install => ActiveBlock::InstallModList,
            };
        }
        _ => {}
    }
}

async fn do_install(
    mod_name: &str,
    factorio_version: &str,
    username: &str,
    token: &str,
    mods_dir: &str,
    installed_map: &HashMap<String, semver::Version>,
) -> anyhow::Result<InstallResult> {
    // Resolve dependencies
    let fetch = |name: String| async move { api::fetch_mod_details(&name).await };
    let resolve_result =
        resolver::resolve(mod_name, factorio_version, installed_map, &fetch).await?;

    let total = resolve_result.to_download.len();
    let dep_count = if total > 0 { total - 1 } else { 0 };

    // Download each mod sequentially
    for (i, (_name, release)) in resolve_result.to_download.iter().enumerate() {
        if let Err(e) = api::download_mod(release, username, token, mods_dir).await {
            // Clean up partial download is handled inside download_mod
            return Err(anyhow::anyhow!(
                "Failed to download '{}' ({}/{} downloaded before failure): {}",
                _name,
                i,
                total,
                e
            ));
        }
    }

    // Re-read installed mods
    let installed_mods = installed::read_installed_mods(mods_dir)?;

    Ok(InstallResult {
        mod_name: mod_name.to_string(),
        dependency_count: dep_count,
        installed_mods,
    })
}


