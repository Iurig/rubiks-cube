use rubiks_cube::Cube3x3;

use crate::message::Message;

#[derive(Debug)]
pub struct App {
    pub mode: Mode,
    pub cube: Cube3x3,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Mode {
    Untimed,
    Timed,
    Exited,
}

impl App {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::ExitProgram => self.mode = Mode::Exited,
        }
    }
}
