use std::{
    fmt::Display,
    fs::{self},
    io::ErrorKind,
    str::FromStr,
};

use bimap::BiHashMap;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use dirs::config_dir;
use rubiks_cube::Move3x3;
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Keybinds {
    pub moves: BiHashMap<KeyPress, Move3x3>,
}

#[derive(Debug, Clone, Copy, SerializeDisplay, DeserializeFromStr, PartialEq, Eq, Hash)]
pub struct KeyPress(KeyCode, KeyModifiers);

impl KeyPress {
    #[must_use]
    pub const fn _new(key: KeyCode) -> Self {
        Self(key, KeyModifiers::empty())
    }

    #[must_use]
    pub const fn with_modifiers(key: KeyCode, modifiers: KeyModifiers) -> Self {
        Self(key, modifiers)
    }

    #[must_use]
    pub const fn code(&self) -> KeyCode {
        self.0
    }

    #[must_use]
    pub const fn modifiers(&self) -> KeyModifiers {
        self.1
    }

    #[must_use]
    pub fn try_from_move(m: Move3x3, keybinds: &Keybinds) -> Option<Self> {
        keybinds.moves.get_by_right(&m).copied()
    }
}

impl Display for KeyPress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.modifiers() == KeyModifiers::empty() {
            write!(f, "{}", self.code())?;
        } else {
            write!(f, "{}+{}", self.modifiers(), self.code())?;
        }
        Ok(())
    }
}

impl From<KeyPress> for KeyEvent {
    fn from(value: KeyPress) -> Self {
        Self {
            code: value.code(),
            modifiers: value.modifiers(),
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
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
                "Ctrl" | "Control" => Ok(KeyModifiers::CONTROL),
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
                'l' => "D'",
                'u' => "r",
                'm' => "r'",
                'v' => "l",
                'r' => "l",
                '/' => "M",
                '[' => "M'"
            )),
        }
    }
}

impl Keybinds {
    #[must_use]
    pub fn as_move(&self, key_press: &KeyPress) -> Option<Move3x3> {
        self.moves.get_by_left(key_press).copied()
    }

    pub fn from_config() -> anyhow::Result<Self> {
        let dir = config_dir()
            .map(|dir| dir.join("virtual_cube_timer"))
            .expect("manually written");

        fs::create_dir_all(dir.clone())?;

        let keybinds_path = dir.join("keybinds.json");

        match std::fs::read_to_string(&keybinds_path) {
            Ok(text) => serde_json::from_str::<Self>(&text).map_err(anyhow::Error::from),
            Err(e) if e.kind() == ErrorKind::NotFound => {
                fs::write(
                    keybinds_path,
                    serde_json::to_string_pretty(&Self::default())?,
                )?;
                Ok(Self::default())
            }
            Err(e) => Err(e.into()),
        }
    }
}

#[must_use]
pub const fn key_press(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::empty())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn default_survives_round_trip() {
        let keybinds = Keybinds::default();
        assert_eq!(
            keybinds,
            serde_json::from_str(&serde_json::to_string_pretty(&keybinds).unwrap()).unwrap()
        );
    }

    #[test]
    fn default_with_ctrl_alt_or_shift_survives_round_trip() {
        let default = Keybinds::default();

        let modif_fn = |index: usize| Keybinds {
            moves: [
                KeyModifiers::CONTROL,
                KeyModifiers::ALT,
                KeyModifiers::SHIFT,
                KeyModifiers::CONTROL
                    .union(KeyModifiers::ALT)
                    .union(KeyModifiers::SHIFT),
            ]
            .iter()
            .map(|modif| {
                default
                    .moves
                    .iter()
                    .map(|(key, m)| (KeyPress(key.0, *modif), *m))
                    .collect::<BiHashMap<_, _>>()
            })
            .nth(index)
            .unwrap(),
        };

        let [ctrl, alt, shift, all] = [modif_fn(0), modif_fn(1), modif_fn(2), modif_fn(3)];

        let round_trip = |bind: Keybinds| -> Keybinds {
            serde_json::from_str(&serde_json::to_string_pretty(&bind).map_or_else(
                |e| panic!("{e}"),
                |s| {
                    println!("{s}");
                    s
                },
            ))
            .map_or_else(
                |e| panic!("{e}"),
                |s| {
                    dbg!(&s);
                    s
                },
            )
        };

        assert_eq!(ctrl.clone(), round_trip(ctrl));
        assert_eq!(alt.clone(), round_trip(alt));
        assert_eq!(shift.clone(), round_trip(shift));
        assert_eq!(all.clone(), round_trip(all));
    }
}
