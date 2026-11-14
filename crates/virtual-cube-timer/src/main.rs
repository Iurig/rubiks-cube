mod app;
mod event;
mod keybinds;
mod message;
mod solves;
mod view;

use std::time::{Duration, Instant};

use anyhow::{Ok, ensure};
use clap::{Parser, ValueEnum};
use ratatui_core as _;
use rubiks_cube::{
    Cube3x3, Puzzle,
    roux::{FirstBlock, SecondBlock},
};

use crate::{
    app::{
        App, Mode,
        navigation::{NavigationApp, NavigationMode},
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

#[derive(Debug, Clone, ValueEnum, PartialEq, Eq)]
enum ValidMethods {
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
                solves: SolveList::from_solves_file()?,
                starting_time: Instant::now(),
                timer_number: Duration::ZERO,
                timer_state: app::timing::TimerState::Solved,
            }),

            _mode => Mode::Navigation(NavigationApp {
                solve_index: 0,
                all_recons: match args.method {
                    ValidMethods::Roux => reconstruct_n(
                        args.number.unwrap_or(30),
                        &<rubiks_cube::Roux as rubiks_cube::Method<Cube3x3>>::new()
                            .first_block(FirstBlock::SquarePair)
                            .second_block(SecondBlock::EdgePairPair),
                    )?,
                    ValidMethods::Zz => reconstruct_n(
                        args.number
                            .expect("demo mode must have a number paramenter"),
                        &<rubiks_cube::ZZ as rubiks_cube::Method<Cube3x3>>::new(),
                    )?,
                },
                nav_mode: NavigationMode::Solves,
            }),
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

                Mode::Navigation(ref mut navigation_app) => {
                    terminal.draw(|frame| render_navigation(navigation_app, frame))?;

                    crossterm::event::read()?
                        .as_key_press_event()
                        .map_or_default(|event| navigation_app.update_with_message(event))
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
