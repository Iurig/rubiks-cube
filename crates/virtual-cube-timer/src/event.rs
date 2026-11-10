use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use crossterm::event::KeyEvent;

#[derive(Debug, Clone, Copy)]
pub enum Event {
    Tick,
    Key(KeyEvent),
}

#[derive(Debug)]
pub struct EventHandler {
    #[allow(dead_code)]
    sender: mpsc::Sender<Event>,
    receiver: mpsc::Receiver<Event>,
    #[allow(dead_code)]
    handler: thread::JoinHandle<()>,
}

impl EventHandler {
    pub fn new(tick_length: Duration) -> Self {
        let (sender, receiver) = mpsc::channel();
        let handler = {
            let sender = sender.clone();
            thread::spawn(move || {
                let mut last_tick = Instant::now();
                loop {
                    let remaining_time = tick_length.saturating_sub(last_tick.elapsed());

                    if crossterm::event::poll(remaining_time).expect("event polling failed") {
                        crossterm::event::read()
                            .expect("event couldn't be read")
                            .as_key_press_event().map(|event| sender.send(Event::Key(event)));
                    } else {
                        sender.send(Event::Tick).expect("failed to send tick event");
                        last_tick = Instant::now();
                    }
                }
            })
        };

        Self {
            sender,
            receiver,
            handler,
        }
    }

    pub fn next(&self) -> anyhow::Result<Event> {
        Ok(self.receiver.recv()?)
    }
}
