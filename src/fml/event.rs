use std::time::Duration;

use crossterm::event::KeyModifiers;
use futures_util::StreamExt;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub tick_rate: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tick_rate: Duration::from_millis(250),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Event<I> {
    Input(I),
    Tick,
}

#[derive(Debug)]
pub enum KeyCode {
    Char(char),
    Ctrl(char),
    Tab,
    Up,
    Down,
    Enter,
    Backspace,
    Unknown,
}

pub struct Events {
    rx: mpsc::UnboundedReceiver<Event<KeyCode>>,
}

impl Events {
    pub fn with_config(config: Option<Config>) -> Self {
        let config = config.unwrap_or_default();
        let (tx, rx) = mpsc::unbounded_channel();
        let tx = tx.clone();
        let mut event_stream = crossterm::event::EventStream::new();

        tokio::spawn(async move {
            loop {
                let event = event_stream.next();
                let delay = tokio::time::sleep(config.tick_rate);

                tokio::select! {
                  _ = delay => {
                    tx.send(Event::Tick).unwrap();
                  },
                  maybe_event = event => {
                    match maybe_event {
                      Some(Ok(event)) => {
                        if let crossterm::event::Event::Key(key_event) = event {
                          let key_code: KeyCode = match key_event.code {
                            crossterm::event::KeyCode::Up => KeyCode::Up,
                            crossterm::event::KeyCode::Down => KeyCode::Down,
                            crossterm::event::KeyCode::Enter => KeyCode::Enter,
                            crossterm::event::KeyCode::Backspace => KeyCode::Backspace,
                            crossterm::event::KeyCode::Tab => KeyCode::Tab,
                            crossterm::event::KeyCode::Char(c) => match key_event.modifiers {
                              KeyModifiers::CONTROL => KeyCode::Ctrl(c),
                              _ => KeyCode::Char(c)
                            },
                            _ => KeyCode::Unknown
                          };
                          tx.send(Event::Input(key_code)).unwrap();
                        }
                      }
                      Some(Err(e)) => {
                        println!("Error: {:?}", e);
                      }
                      None => break
                    }
                  }
                }
            }
        });

        Self { rx }
    }

    pub fn next(&mut self) -> impl futures_util::Future<Output = Option<Event<KeyCode>>> + '_ {
        self.rx.recv()
    }
}
