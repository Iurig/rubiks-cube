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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Keybinds {
    moves: BiHashMap<KeyPress, Move3x3>,
}

#[derive(Debug, Clone, Copy, SerializeDisplay, DeserializeFromStr, PartialEq, Eq, Hash)]
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
        if self.1 == KeyModifiers::empty() {
            write!(f, "{}", self.0)?;
        } else {
            write!(f, "{}+{}", self.1, self.0)?;
        }
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

#[cfg(test)]
mod test {

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
            serde_json::from_str(
                &serde_json::to_string_pretty(&bind)
                    .map(|s| {
                        println!("{s}");
                        s
                    })
                    .unwrap_or_else(|e| panic!("{e}")),
            )
            .map(|s| {
                dbg!(&s);
                s
            })
            .unwrap_or_else(|e| panic!("{e}"))
        };

        assert_eq!(ctrl.clone(), round_trip(ctrl));
        assert_eq!(alt.clone(), round_trip(alt));
        assert_eq!(shift.clone(), round_trip(shift));
        assert_eq!(all.clone(), round_trip(all));
    }
}
