mod app;
mod event;
mod keybinds;
mod message;
mod solves;
mod view;

use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use anyhow::{Ok, ensure};
use clap::{Parser, ValueEnum};
use ratatui_core as _;
use rubiks_cube::{
    Algorithm, Cube3x3, Method, Puzzle, Roux, Solution, ZZ,
    roux::{FirstBlock, SecondBlock},
};

use crate::{
    app::{
        App, Mode,
        navigation::{ListNavApp, NavigationApp},
        timing::TimingApp,
        untimed::UntimedApp,
    },
    event::EventHandler,
    keybinds::Keybinds,
    message::Message,
    solves::{ReconWithScramble, SolveList},
    view::{navigation::render_navigation, timing::render_timing, untimed::render_untimed},
};

#[derive(Debug, Parser, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(value_enum)]
    method: ValidMethods,

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

#[derive(Debug, Default, Clone, ValueEnum, PartialEq, Eq)]
enum ValidMethods {
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
                method: args.method.clone(),
                solves: Arc::new(Mutex::new(SolveList::from_solves_file()?)),
                curr_solve_scramble: VecDeque::new(),
                curr_solve_moves: Solution::single_segment(args.method.name(), Algorithm::new()),
                starting_time: Instant::now(),
                timer_number: Duration::ZERO,
                timer_state: app::timing::TimerState::Solved,
            }),

            _mode => Mode::Nav(NavigationApp::List(ListNavApp {
                solve_index: 0,
                all_recons: match args.method {
                    ValidMethods::Roux => reconstruct_n(
                        args.number.unwrap_or(30),
                        &<rubiks_cube::Roux as rubiks_cube::Method<Cube3x3>>::new()
                            .first_block(FirstBlock::SquarePair)
                            .second_block(SecondBlock::EdgePairPair),
                    )?,
                    ValidMethods::Zz => reconstruct_n(
                        args.number.expect("demo mode must have a number parameter"),
                        &<rubiks_cube::ZZ as rubiks_cube::Method<Cube3x3>>::new(),
                    )?,
                },
            })),
        },
    };

    ratatui::run(|terminal| -> anyhow::Result<()> {
        let event_handler = EventHandler::new(Duration::from_millis(5));
        loop {
            let message = match app.mode {
                Mode::Timing(ref mut timing_app) => {
                    terminal.draw(|frame| render_timing(timing_app, frame))?;

                    timing_app.update_with_message(event_handler.next()?, &keybinds)
                }

                Mode::Untimed(ref mut untimed_app) => {
                    terminal.draw(|frame| render_untimed(untimed_app, frame))?;

                    crossterm::event::read()?
                        .as_key_press_event()
                        .map_or_default(|event| untimed_app.update_with_message(event, &keybinds))
                }

                Mode::Nav(ref mut solve_app) => {
                    terminal.draw(|frame| render_navigation(solve_app, frame))?;

                    crossterm::event::read()?
                        .as_key_press_event()
                        .map_or_default(|event| solve_app.update_with_message(event))
                }
            };
            match message {
                Some(Message::ExitProgram) => break,
                None => (),
            }
        }
        Ok(())
    })?;
    Ok(())
}

fn reconstruct_n<T: rubiks_cube::Method<Cube3x3>>(
    n: u32,
    technique: &T,
) -> anyhow::Result<Vec<ReconWithScramble>> {
    let mut recons = Vec::new();
    for _ in 0..n {
        let scramble = Cube3x3::scramble()?;

        let mut puzzle = Cube3x3::default().apply(&scramble);

        let solution = technique.solve(&mut puzzle)?;

        ensure!(
            puzzle.is_solved(),
            "scramble:\n {scramble} \n and solve:\n {solution} \n did not solve the cube"
        );
        recons.push(ReconWithScramble::new(scramble, solution));
    }
    Ok(recons)
}
