pub mod app;
pub mod event;
pub mod keybinds;
pub mod message;
pub mod solves;
pub mod time;
pub mod view;

use clap::{Parser, ValueEnum};
use rubiks_cube::{Cube3x3, Method, Roux, ZZ};

#[derive(Debug, Parser, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(value_enum)]
    pub method: ValidMethods,

    #[arg(value_enum)]
    pub mode: ModeArg,

    #[arg(short, long)]
    pub number: Option<u32>,
}

#[derive(Debug, Clone, ValueEnum, Default)]
pub enum ModeArg {
    #[default]
    Untimed,
    Normal,
    Demo,
    Browse,
}

#[derive(Debug, Default, Clone, ValueEnum, PartialEq, Eq)]
pub enum ValidMethods {
    #[default]
    Roux,
    Zz,
}

impl Method<Cube3x3> for ValidMethods {
    fn new() -> Self {
        Self::default()
    }

    fn to_technique(&self) -> rubiks_cube::Technique<Cube3x3> {
        match self {
            Self::Roux => Roux::default().to_technique(),
            Self::Zz => ZZ::default().to_technique(),
        }
    }
}

impl ValidMethods {
    pub const fn name(&self) -> &str {
        match self {
            Self::Roux => "Roux",
            Self::Zz => "ZZ",
        }
    }
}
