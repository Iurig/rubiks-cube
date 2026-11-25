use crossterm::event::{KeyCode, KeyEvent};
use rubiks_cube::Cube3x3;

use crate::{
    keybinds::{KeyPress, Keybinds},
    message::{CubeMessage, Message},
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct UntimedApp {
    pub cube: Cube3x3,
    pub keybinds: Keybinds,
}

impl UntimedApp {
    #[must_use]
    pub fn new(cube: Cube3x3) -> Self {
        Self {
            cube,
            keybinds: Keybinds::try_from_config().unwrap_or_default(),
        }
    }

    pub fn update(&mut self, message: &CubeMessage) {
        match message {
            CubeMessage::Apply(m) => self.cube *= *m,
        }
    }

    pub fn update_with_message(&mut self, event: KeyEvent) -> Option<Message> {
        let KeyEvent {
            code,
            modifiers,
            kind: _,
            state: _,
        } = event;

        match (code, modifiers) {
            (KeyCode::Esc, _) => return Some(Message::ExitProgram),
            _ if let Some(m) = self
                .keybinds
                .as_move(&KeyPress::with_modifiers(code, modifiers)) =>
            {
                self.update(&CubeMessage::Apply(m));
            }
            _ => (),
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::KeyModifiers;
    use rubiks_cube::{Algorithm, FromAlg};

    use super::*;

    #[test]
    fn keybinds_test() {
        let mut app = UntimedApp::new(Cube3x3::default());

        let sexy = Algorithm::<Cube3x3>::from_static("R U R' U'");

        for m in &sexy {
            assert_eq!(
                app.update_with_message(KeyEvent::from(
                    KeyPress::try_from_move(*m, &Keybinds::from_config()).unwrap(),
                )),
                None
            );
        }

        assert_eq!(app.cube, Cube3x3::from_alg(&sexy));

        assert_eq!(
            app.update_with_message(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
            Some(Message::ExitProgram)
        );
    }
}
