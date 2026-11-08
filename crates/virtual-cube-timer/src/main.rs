mod app;
mod keybinds;
mod message;
mod view;

use clap::{Parser, ValueEnum};
use crossterm::event::{self, KeyCode, KeyEvent};
use ratatui_core as _;
use rubiks_cube::Cube3x3;

use crate::{
    app::{App, Mode},
    keybinds::{KeyPress, Keybinds},
    message::Message,
    view::render_view,
};

#[derive(Debug, Parser, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(value_enum)]
    method: Methods,

    #[arg(long, short)]
    timed: bool,
}

#[derive(Debug, Clone, ValueEnum)]
enum Methods {
    Roux,
    Zz,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let keybinds = Keybinds::from_config()?;

    if args.timed {
        let _app = App {
            mode: Mode::Timed,
            cube: Cube3x3::default(),
        };
    } else {
        let mut app = App {
            mode: Mode::Untimed,
            cube: Cube3x3::default(),
        };

        ratatui::run(|terminal| -> anyhow::Result<()> {
            while app.mode != Mode::Exited {
                terminal.draw(|frame| render_view(&app, frame))?;

                if let Some(KeyEvent {
                    code: key,
                    modifiers: modif,
                    kind: _,
                    state: _,
                }) = event::read()?.as_key_press_event()
                {
                    match (key, modif) {
                        (KeyCode::Esc, _) => app.update(&Message::ExitProgram),
                        _ if let Some(m) =
                            keybinds.as_move(&KeyPress::with_modifiers(key, modif)) =>
                        {
                            app.update(&Message::ApplyMove(m));
                        }
                        _ => (),
                    }
                }
            }

            Ok(())
        })?;
    }
    Ok(())
}
