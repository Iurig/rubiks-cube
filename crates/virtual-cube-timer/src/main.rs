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

    if !args.timed {
        let mut app = App {
            mode: Mode::Untimed,
            cube: Cube3x3::default(),
        };

        ratatui::run(|terminal| -> anyhow::Result<()> {
            while app.mode != Mode::Exited {
                terminal.draw(|frame| render_view(&app, frame))?;

                if let Some(KeyEvent {
                    code: key,
                    modifiers: _,
                    kind: _,
                    state: _,
                }) = event::read()?.as_key_press_event()
                {
                    match key {
                        KeyCode::Esc => app.update(Message::ExitProgram),
                        _ => (),
                    }
                }
            }

            Ok(())
        })?;
    }
    Ok(())
}
