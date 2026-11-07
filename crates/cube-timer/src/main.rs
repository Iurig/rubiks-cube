mod app;
mod view;

use clap::{Parser, ValueEnum};
use crossterm::event::{self, KeyEvent};
use rubiks_cube::Cube3x3;

use crate::{
    app::{App, Mode},
    view::render_view,
};

#[derive(Debug, Parser, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(value_enum)]
    method: Methods,

    #[arg(long, short)]
    untimed: bool,
}

#[derive(Debug, Clone, ValueEnum)]
enum Methods {
    Roux,
    Zz,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if args.untimed {
        let mut app = App {
            mode: Mode::Untimed,
            cube: Cube3x3::default(),
        };

        ratatui::run(|terminal| -> anyhow::Result<()> {
            while app.mode != Mode::Exited {
                terminal.draw(|frame| render_view(&app, frame))?;
            }

            if let Some(KeyEvent {
                code: _,
                modifiers: _,
                kind: _,
                state: _,
            }) = event::read()?.as_key_press_event()
            {
                todo!()
            }

            Ok(())
        })?;
    }
    Ok(())
}
