use bimap::BiHashMap;
use crossterm::event::{KeyCode, KeyModifiers};
use rubiks_cube::Move3x3;

#[derive(Debug)]
pub struct Keybinds {
    moves: BiHashMap<(KeyCode, KeyModifiers), Move3x3>,
}
