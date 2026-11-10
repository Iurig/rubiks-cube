use crossterm::event::{KeyCode, KeyEvent};
use rubiks_cube::Cube3x3;

use crate::{
    keybinds::{KeyPress, Keybinds},
    message::{CubeMessage, Message},
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct UntimedApp {
    pub cube: Cube3x3,
}

impl UntimedApp {
    pub fn update(&mut self, message: &CubeMessage) {
        match message {
            CubeMessage::Apply(m) => self.cube *= *m,
        }
    }

    pub fn update_with_message(&mut self, event: KeyEvent, keybinds: &Keybinds) -> Option<Message> {
        let KeyEvent {
            code,
            modifiers,
            kind: _,
            state: _,
        } = event;

        match (code, modifiers) {
            (KeyCode::Esc, _) => return Some(Message::ExitProgram),
            _ if let Some(m) = keybinds.as_move(&KeyPress::with_modifiers(code, modifiers)) => {
                self.update(&CubeMessage::Apply(m));
            }
            _ => (),
        }
        None
    }
}
