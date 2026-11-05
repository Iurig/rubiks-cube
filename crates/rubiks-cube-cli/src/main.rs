mod app;
mod message;
mod view;

use anyhow::ensure;
use clap::{Parser, ValueEnum};
use crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers};
use rubiks_cube::{Method, Puzzle, Roux, ZZ};

use crate::app::{App, Mode};
use crate::message::{BrowseMessage, InspectMessage, Message};
use crate::view::view;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(value_enum)]
    method: Methods,

    #[arg(long)]
    number: u32,

    #[arg(long)]
    iterative: bool,
}

#[derive(Debug, Clone, ValueEnum)]
enum Methods {
    Roux,
    Zz,
}

fn reconstruct_n<P: Puzzle, T: Method<P>>(n: u32, technique: &T) -> anyhow::Result<Vec<String>> {
    let mut recons = Vec::new();
    for _ in 0..n {
        let scramble = P::scramble()?;
        let scramble_text = scramble.to_string().replace("2'", "2");

        let mut puzzle = P::default().apply(&scramble);

        let solution = technique.solve(&mut puzzle)?;

        ensure!(
            puzzle.is_solved(),
            "scramble:\n {scramble_text} \n and solve:\n {solution} \n did not solve the cube"
        );

        recons.push(format!("{scramble_text}\n{solution}"));
    }
    Ok(recons)
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let recons = match args.method {
        Methods::Roux => reconstruct_n(args.number, &Roux::default())?,
        Methods::Zz => reconstruct_n(args.number, &ZZ::default())?,
    };

    if args.iterative {
        let mut app = App {
            solve_index: 0,
            all_recons: recons,
            mode: Mode::default(),
            mode_info: app::ModeInfo::BrowseSolves,
        };
        ratatui::run(|terminal| -> anyhow::Result<()> {
            while app.mode != Mode::Exited {
                terminal.draw(|frame| view(&app, frame))?;
                if let Some(KeyEvent {
                    code: pressed_key,
                    modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                    kind: _,
                    state: _,
                }) = event::read()?.as_key_press_event()
                {
                    match app.mode {
                        Mode::BrowseSolves => match pressed_key {
                            KeyCode::Esc => app.update(Message::ExitProgram),

                            KeyCode::Char('p') | KeyCode::Char('P') => {
                                app.update(Message::BrowseSolves(BrowseMessage::PreviousSolve));
                            }

                            KeyCode::Char('n') | KeyCode::Char('N') => {
                                app.update(Message::BrowseSolves(BrowseMessage::NextSolve))
                            }

                            KeyCode::Enter => {
                                app.update(Message::BrowseSolves(BrowseMessage::EnterInspect))
                            }

                            _ => (),
                        },

                        Mode::Inspect => match pressed_key {
                            KeyCode::Esc => app.update(Message::Inspect(InspectMessage::Exit)),

                            KeyCode::Char('p') | KeyCode::Char('P') => {
                                app.update(Message::Inspect(InspectMessage::PreviousMove))
                            }

                            KeyCode::Char('n') | KeyCode::Char('N') => {
                                app.update(Message::Inspect(InspectMessage::NextMove))
                            }

                            _ => (),
                        },

                        Mode::Exited => (),
                    }
                }
            }
            Ok(())
        })?;
    } else {
        for (i, solve) in recons.into_iter().enumerate() {
            println!("{}. {solve}", i + 1);
        }
    }
    Ok(())
}
