use std::{any, ops::ControlFlow};

use anyhow::ensure;
use clap::Parser;
use crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers};
use ratatui::Frame;
use rubiks_cube::{Method, Puzzle, Roux, ZZ};
use thiserror::Error;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    method: String,

    #[arg(long)]
    number: u32,

    #[arg(long)]
    iterative: bool,
}

fn reconstruct_n<P: Puzzle, T: Method<P>>(n: u32, technique: &T) -> anyhow::Result<Vec<String>> {
    let mut recons = Vec::new();
    for _ in 0..n {
        let scramble = P::scramble()?;

        let mut puzzle = P::default().apply(&scramble);

        let solution = technique.solve(&mut puzzle)?;

        ensure!(
            puzzle.is_solved(),
            "scramble:\n {scramble} \n and solve:\n {solution} \n did not solve the cube"
        );

        recons.push(format!("{scramble} {solution}"));
    }
    Ok(recons)
}

fn walkthrough_recon(recon: &str, terminal: &mut Frame) {
    terminal.render_widget("Testing.", terminal.area());
}

#[derive(Debug, Error)]
enum StopIteration {
    #[error("the iteration was stopped")]
    ExitProgram,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let recon = match args.method.as_str() {
        "roux" => reconstruct_n(args.number, &Roux::default())?,
        "zz" => reconstruct_n(args.number, &ZZ::default())?,
        _ => vec!["invalid method".to_owned()],
    };

    if args.iterative {
        let mut index = 0;
        ratatui::run(|terminal| -> anyhow::Result<()> {
            loop {
                terminal.draw(|frame| walkthrough_recon(&recon[index], frame))?;
                if let Some(KeyEvent {
                    code: pressed_key,
                    modifiers: KeyModifiers::NONE,
                    kind: _,
                    state: _,
                }) = event::read()?.as_key_press_event()
                {
                    match pressed_key {
                        KeyCode::Esc => {
                            break Err(anyhow::Error::new(StopIteration::ExitProgram));
                        }
                        KeyCode::Left => {
                            index -= 1;
                            index = std::cmp::max(index, 0);
                        }
                        KeyCode::Right => {
                            index += 1;
                            index = std::cmp::min(index, recon.len() - 1);
                        }
                        _ => continue,
                    }
                }
            }
        })?;
        Ok(())
    } else {
        for (i, solve) in recon.into_iter().enumerate() {
            println!("{}. {solve}", i + 1);
        }
        Ok(())
    }
}
