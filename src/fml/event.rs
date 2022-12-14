use std::sync::mpsc;
use std::time::Duration;

use crossterm::event::KeyModifiers;
use futures_util::StreamExt;

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

#[derive(Debug, Clone)]
pub enum KeyCode {
    Char(char),
    Ctrl(char),
    Tab,
    Up,
    Down,
    Right,
    Left,
    Enter,
    Backspace,
    Esc,
    Unknown,
}

pub struct Events {
    rx: mpsc::Receiver<Event<KeyCode>>,
    pub tx: mpsc::Sender<Event<KeyCode>>,
}

impl Events {
    pub fn with_config(config: Option<Config>) -> Self {
        let config = config.unwrap_or_default();
        let (tx, rx) = mpsc::channel();
        let tx_ = tx.clone();
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
                            crossterm::event::KeyCode::Left => KeyCode::Left,
                            crossterm::event::KeyCode::Right => KeyCode::Right,
                            crossterm::event::KeyCode::Enter => KeyCode::Enter,
                            crossterm::event::KeyCode::Backspace => KeyCode::Backspace,
                            crossterm::event::KeyCode::Tab => KeyCode::Tab,
                            crossterm::event::KeyCode::Esc => KeyCode::Esc,
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

        Self { rx, tx: tx_ }
    }

    pub fn next(&mut self) -> Result<Event<KeyCode>, mpsc::RecvError> {
        self.rx.recv()
    }
}
