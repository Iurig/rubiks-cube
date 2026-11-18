#![warn(unused_crate_dependencies)]

mod app;
mod event;
mod keybinds;
mod message;
mod solves;
mod time;
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
    Cube3x3, Method, Puzzle, Roux, Solution, ZZ,
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
    solves::{ReconWithScramble, SolveList, TimedSolve},
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

    let solves = SolveList::from_solves_file(SolveList::solves_path()?)?;

    let mut app = App {
        mode: match args.mode {
            ModeArg::Untimed => Mode::Untimed(UntimedApp {
                cube: Cube3x3::default(),
                keybinds,
            }),
            ModeArg::Normal => Mode::Timing(TimingApp {
                cube: Cube3x3::default(),
                technique: args.method.to_technique(),
                keybinds,
                step_number: 0,
                solves: Arc::new(Mutex::new(solves)),
                curr_solve_scramble: VecDeque::new(),
                curr_solve_moves: Solution::with_empty_steps(
                    args.method
                        .to_technique()
                        .steps()
                        .map(rubiks_cube::Step::name),
                ),
                starting_time: Instant::now(),
                timer_number: Duration::ZERO,
                timer_state: app::timing::TimerState::Solved,
            }),

            _mode => Mode::Nav(NavigationApp::List(ListNavApp {
                solve_index: 0,
                all_recons: reconstruct_n(args.number.unwrap_or(30), &args.method)?,
            })),
        },
    };

    ratatui::run(|terminal| -> anyhow::Result<()> {
        let event_handler = EventHandler::new(Duration::from_millis(5));
        loop {
            let message = match app.mode {
                Mode::Timing(ref mut timing_app) => {
                    terminal.draw(|frame| render_timing(timing_app, frame))?;

                    timing_app.update_with_message(event_handler.next()?)
                }

                Mode::Untimed(ref mut untimed_app) => {
                    terminal.draw(|frame| render_untimed(untimed_app, frame))?;

                    crossterm::event::read()?
                        .as_key_press_event()
                        .map_or_default(|event| untimed_app.update_with_message(event))
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

fn reconstruct_n(n: u32, method: &ValidMethods) -> anyhow::Result<SolveList> {
    let mut recons = Vec::new();
    let technique = match method {
        ValidMethods::Roux => &<rubiks_cube::Roux as rubiks_cube::Method<Cube3x3>>::new()
            .first_block(FirstBlock::SquarePair)
            .second_block(SecondBlock::EdgePairPair)
            .to_technique(),
        ValidMethods::Zz => {
            &<rubiks_cube::ZZ as rubiks_cube::Method<Cube3x3>>::new().to_technique()
        }
    };
    for _ in 0..n {
        let scramble = Cube3x3::scramble()?;

        let mut puzzle = Cube3x3::default().apply(&scramble);

        let solution = technique.solve(&mut puzzle)?;

        ensure!(
            puzzle.is_solved(),
            "scramble:\n {scramble} \n and solve:\n {solution} \n did not solve the cube"
        );
        recons.push(TimedSolve::new(
            ReconWithScramble::new(scramble, solution),
            Duration::ZERO,
            method.clone(),
        ));
    }
    Ok(SolveList::from_solves(recons))
}
