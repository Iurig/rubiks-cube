pub mod navigation;
pub mod timing;
pub mod untimed;

use crate::{
    app::{navigation::NavigationApp, timing::TimingApp, untimed::UntimedApp},
    message::Message::{self, ExitProgram},
};

#[derive(Debug, Clone)]
pub struct App {
    pub mode: Mode,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Mode {
    Untimed(UntimedApp),
    Timing(TimingApp),
    Navigation(NavigationApp),
    Exited,
}

impl App {
    pub fn update(&mut self, message: &Message) {
        match message {
            ExitProgram => self.mode = Mode::Exited,
        }
    }

    pub fn is_running(&self) -> bool {
        if self.mode == Mode::Exited {
            false
        } else {
            true
        }
    }
}
