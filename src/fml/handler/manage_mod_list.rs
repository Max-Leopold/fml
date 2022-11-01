use crate::fml::app::FML;
use crate::fml::event::{Event, KeyCode};

use super::handler::EventHandler;

pub struct ManageModListHandler {}

impl EventHandler for ManageModListHandler {
    fn handle(event: Event<KeyCode>, app: &mut FML) {
        match event {
            _ => {}
        }
    }
}
