use anyhow::Result;
use crossterm::event::{self, Event, KeyEvent};
use std::time::Duration;
use tokio::sync::mpsc;

use crate::factorio::installed::InstalledMod;
use crate::factorio::mod_list::ModList;
use crate::factorio::types::ModListEntry;

#[derive(Debug)]
pub enum AppEvent {
    Key(KeyEvent),
    Tick,
    ModListLoaded(Result<Vec<ModListEntry>>),
    ModInstalled(Result<InstallResult>),
    ModDeleted(Result<String>),
    InstalledModsLoaded(Result<(Vec<InstalledMod>, ModList)>),
    Error(String),
}

#[derive(Debug)]
pub struct InstallResult {
    pub mod_name: String,
    pub dependency_count: usize,
    pub installed_mods: Vec<InstalledMod>,
}

pub fn spawn_event_loop(tx: mpsc::UnboundedSender<AppEvent>) {
    tokio::spawn(async move {
        let tick_rate = Duration::from_millis(250);
        loop {
            if event::poll(tick_rate).unwrap_or(false) {
                if let Ok(Event::Key(key)) = event::read() {
                    if tx.send(AppEvent::Key(key)).is_err() {
                        break;
                    }
                }
            } else if tx.send(AppEvent::Tick).is_err() {
                break;
            }
        }
    });
}
