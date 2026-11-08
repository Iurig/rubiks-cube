use std::{
    fmt::Display,
    fs::{self},
    io::ErrorKind,
    str::FromStr,
};

use bimap::BiHashMap;
use crossterm::event::{KeyCode, KeyModifiers};
use dirs::config_dir;
use rubiks_cube::Move3x3;
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};

#[derive(Debug, Serialize, Deserialize)]
pub struct Keybinds {
    moves: BiHashMap<KeyPress, Move3x3>,
}

#[derive(Debug, SerializeDisplay, DeserializeFromStr, PartialEq, Eq, Hash)]
pub struct KeyPress(KeyCode, KeyModifiers);

impl KeyPress {
    pub const fn _new(key: KeyCode) -> Self {
        Self(key, KeyModifiers::empty())
    }

    pub const fn with_modifiers(key: KeyCode, modifiers: KeyModifiers) -> Self {
        Self(key, modifiers)
    }
}

impl Display for KeyPress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.0, self.1)?;
        Ok(())
    }
}

impl FromStr for KeyPress {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parse_vec = s.split('+').map(str::trim).collect::<Vec<_>>();

        let key = parse_vec.pop().map_or_else(
            || Err(String::from("no key")),
            |key| {
                if let [c] = key.chars().collect::<Vec<_>>()[..] {
                    Ok(KeyCode::Char(c))
                } else {
                    Err(String::from("invalid key"))
                }
            },
        );

        let modifiers = parse_vec
            .iter()
            .map(|&modif| match modif {
                "Shift" => Ok(KeyModifiers::SHIFT),
                "Ctrl" => Ok(KeyModifiers::CONTROL),
                "Alt" => Ok(KeyModifiers::ALT),
                m => Result::Err(format!("invalid modifier {m}")),
            })
            .try_fold(
                KeyModifiers::empty(),
                |modifiers, new_modif| -> Result<KeyModifiers, Self::Err> {
                    Ok(modifiers.union(new_modif?))
                },
            );

        Ok(Self(key?, modifiers?))
    }
}

macro_rules! bind_keys {
    ($($key:literal => $move:literal),+) => {
        [$(
            (KeyPress(KeyCode::Char($key), KeyModifiers::NONE),
            $move.parse().expect("manually typed should parse"))
        ),+]
    };
}

impl Default for Keybinds {
    fn default() -> Self {
        Self {
            moves: BiHashMap::from_iter(bind_keys!(
                'i' => "R",
                'k' => "R'",
                'j' => "U",
                'f' => "U'",
                'h' => "F",
                'g' => "F'",
                'w' => "B",
                'o' => "B'",
                'd' => "L",
                'e' => "L'",
                's' => "D",
                'l' => "D'"
            )),
        }
    }
}

impl Keybinds {
    pub fn as_move(&self, key_press: &KeyPress) -> Option<Move3x3> {
        self.moves.get_by_left(key_press).copied()
    }

    pub fn from_config() -> anyhow::Result<Self> {
        let dir = config_dir()
            .map(|dir| dir.join("virtual_cube_timer"))
            .expect("manually writen");

        fs::create_dir_all(dir.clone())?;

        let keybinds_path = dir.join("keybinds.json");

        match std::fs::read_to_string(&keybinds_path) {
            Ok(text) => serde_json::from_str::<Self>(&text).map_err(anyhow::Error::from),
            Err(e) if e.kind() == ErrorKind::NotFound => {
                fs::write(keybinds_path, serde_json::to_string(&Self::default())?)?;
                Ok(Self::default())
            }
            Err(e) => Err(e.into()),
        }
    }
}
