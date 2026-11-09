mod app;
mod keybinds;
mod message;
mod view;

use anyhow::{Ok, ensure};
use clap::{Parser, ValueEnum};
use crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers};
use ratatui_core as _;
use rubiks_cube::{
    Cube3x3, Puzzle,
    roux::{FirstBlock, SecondBlock},
};

use crate::{
    app::{
        App, Mode,
        navigation::{DemoOptions, InspectOptions, NavigationApp, NavigationMode},
        timing::TimingApp,
        untimed::UntimedApp,
    },
    keybinds::{KeyPress, Keybinds},
    message::{CubeMessage, Message, NavigationMessage},
    view::{navigation::render_navigation, untimed::render_untimed},
};

#[derive(Debug, Parser, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(value_enum)]
    method: Method,

    #[arg(value_enum)]
    mode: ModeArg,

    #[arg(short, long)]
    number: Option<u32>,
}

#[derive(Debug, Clone, ValueEnum, Default)]
enum ModeArg {
    #[default]
    Untimed,
    Normal,
    Demo,
    Browse,
}

#[derive(Debug, Clone, ValueEnum, PartialEq, Eq)]
enum Method {
    Roux,
    Zz,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let keybinds = Keybinds::from_config()?;

    let mut app = App {
        mode: match args.mode {
            ModeArg::Untimed => Mode::Untimed(UntimedApp {
                cube: Cube3x3::default(),
            }),
            ModeArg::Normal => Mode::Timing(TimingApp {
                cube: Cube3x3::default(),
                method: args.method,
            }),

            mode => Mode::Navigation(NavigationApp {
                solve_index: 0,
                demo: match mode {
                    ModeArg::Browse => None,
                    ModeArg::Demo => Some(DemoOptions {
                        method: args.method.clone(),
                    }),
                    _ => unreachable!(),
                },
                all_recons: match args.method {
                    Method::Roux => reconstruct_n(
                        args.number.expect("demo mode must have a number parameter"),
                        &<rubiks_cube::Roux as rubiks_cube::Method<Cube3x3>>::new()
                            .first_block(FirstBlock::SquarePair)
                            .second_block(SecondBlock::EdgePairPair),
                    )?,
                    Method::Zz => reconstruct_n(
                        args.number
                            .expect("demo mode must have a number paramenter"),
                        &<rubiks_cube::ZZ as rubiks_cube::Method<Cube3x3>>::new(),
                    )?,
                },
                nav_mode: NavigationMode::Solves,
            }),
        },
    };

    while app.is_running() {
        let mut cloned_app = app.clone();
        match cloned_app.mode {
            Mode::Timing(ref method) => {
                todo!("timing mode with {method:?}")
            }

            Mode::Untimed(ref untimed_app_ref) => ratatui::run(|terminal| -> anyhow::Result<()> {
                let mut untimed_app = untimed_app_ref.clone();
                while app.is_running() {
                    terminal.draw(|frame| render_untimed(&untimed_app, frame))?;

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
                                untimed_app.update(&CubeMessage::Apply(m));
                            }
                            _ => (),
                        }
                    }
                }
                Ok(())
            })?,

            Mode::Navigation(ref mut navigation_app) => {
                ratatui::run(|terminal| -> anyhow::Result<()> {
                    while app.is_running() {
                        terminal.draw(|frame| render_navigation(&navigation_app, frame))?;
                        if let Some(KeyEvent {
                            code: pressed_key,
                            modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                            kind: _,
                            state: _,
                        }) = event::read()?.as_key_press_event()
                        {
                            match navigation_app.nav_mode {
                                NavigationMode::Solves => match pressed_key {
                                    KeyCode::Esc => app.update(&Message::ExitProgram),

                                    KeyCode::Char('p' | 'P') => {
                                        navigation_app.navigate(&NavigationMessage::PreviousSolve);
                                    }

                                    KeyCode::Char('n' | 'N') => {
                                        navigation_app.navigate(&NavigationMessage::NextSolve);
                                    }

                                    KeyCode::Enter => {
                                        navigation_app.navigate(&NavigationMessage::EnterInspect);
                                    }

                                    _ => (),
                                },

                                NavigationMode::Inspect(InspectOptions {
                                    move_index: _,
                                    step_index: _,
                                }) => match pressed_key {
                                    KeyCode::Esc => {
                                        navigation_app.navigate(&NavigationMessage::EnterSolves)
                                    }

                                    KeyCode::Left => {
                                        navigation_app.navigate(&NavigationMessage::PreviousMove);
                                    }

                                    KeyCode::Right => {
                                        navigation_app.navigate(&NavigationMessage::NextMove);
                                    }

                                    KeyCode::Up => {
                                        navigation_app.navigate(&NavigationMessage::PreviousStep);
                                    }

                                    KeyCode::Down => {
                                        navigation_app.navigate(&NavigationMessage::NextStep)
                                    }

                                    _ => (),
                                },
                            }
                        }
                    }
                    Ok(())
                })?
            }

            Mode::Exited => {
                panic!("Error, should never enter the loop in exited mode")
            }
        }
    }
    Ok(())
}

fn reconstruct_n<P: Puzzle, T: rubiks_cube::Method<P>>(
    n: u32,
    technique: &T,
) -> anyhow::Result<Vec<String>> {
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
