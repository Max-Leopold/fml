use std::sync::{Arc, Mutex};

use tokio::sync::mpsc;
use tui::widgets::Gauge;

use crate::factorio::api::Dependencies;
use crate::factorio::{api, installed_mods};

use super::install_mod_list::InstallModList;
use super::manage_mod_list::{self, ManageModList};

#[derive(Debug, Clone)]
pub struct ModDownloadRequest {
    pub mod_name: String,
    pub username: String,
    pub token: String,
    pub mod_dir: String,
}

pub struct ModDownloader {
    pub tx: mpsc::UnboundedSender<ModDownloadRequest>,
    download_perc: Arc<Mutex<u16>>,
    currently_downloading: Arc<Mutex<String>>,
}

impl ModDownloader {
    pub fn new(
        install_mod_list: Arc<Mutex<InstallModList>>,
        manage_mod_list: Arc<Mutex<ManageModList>>,
    ) -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<ModDownloadRequest>();
        let tx_ = tx.clone();

        let download_perc = Arc::new(Mutex::new(0));
        let currently_downloading = Arc::new(Mutex::new(String::new()));
        let download_perc_clone = download_perc.clone();
        let currently_downloading_clone = currently_downloading.clone();
        tokio::spawn(async move {
            loop {
                let download_request = rx.recv();
                let debounce = tokio::time::sleep(std::time::Duration::from_millis(250));

                tokio::select! {
                    _ = debounce => {
                        *download_perc_clone.lock().unwrap() = 0;
                        *currently_downloading_clone.lock().unwrap() = String::new();
                    }
                    maybe_download_request = download_request => {
                      match maybe_download_request {
                        Some(download_request) => {
                          // The base mod is always a dependency but it's not actually a mod but instead the base game.
                          // So we just skip it when asked to download because it's not available on the mod portal.
                          if download_request.mod_name == "base" {
                              continue;
                          }
                          let mod_ = api::get_mod(&download_request.mod_name).await.unwrap();
                          *download_perc_clone.lock().unwrap() = 0;
                          *currently_downloading_clone.lock().unwrap() = mod_.title.clone();

                          let mut file = api::download_mod(
                              &mod_,
                              &download_request.username,
                              &download_request.token,
                              &download_request.mod_dir,
                              Some(|x| {
                                  *download_perc_clone.lock().unwrap() = x;
                              }),
                          )
                          .await
                          .unwrap();

                          // Download all mod dependencies
                          for dependency in mod_
                              .latest_release()
                              .info_json
                              .dependencies
                              .unwrap_or(Dependencies::default())
                              .required
                              .iter()
                          {
                              tx.send(ModDownloadRequest {
                                  mod_name: dependency.name.clone(),
                                  username: download_request.username.clone(),
                                  token: download_request.token.clone(),
                                  mod_dir: download_request.mod_dir.clone(),
                              })
                              .unwrap();
                          }

                          install_mod_list
                              .lock()
                              .unwrap()
                              .enable_mod(&download_request.mod_name);
                          if let Some(installed_mod) = installed_mods::parse_installed_mod(&mut file) {
                              manage_mod_list.lock().unwrap().add_mod(installed_mod, true);
                          }
                        }
                        _ => {}
                      }
                    }
                }
            }
        });

        ModDownloader {
            tx: tx_,
            download_perc,
            currently_downloading,
        }
    }

    pub fn generate_gauge(&self) -> Option<Gauge> {
        if self.currently_downloading.lock().unwrap().is_empty() {
            return None;
        }

        let gauge = Gauge::default()
            .percent(self.download_perc.lock().unwrap().clone())
            .label(self.currently_downloading.lock().unwrap().clone());

        Some(gauge)
    }
}
