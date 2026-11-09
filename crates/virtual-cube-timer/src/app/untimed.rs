use rubiks_cube::Cube3x3;

use crate::message::CubeMessage;

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
}
