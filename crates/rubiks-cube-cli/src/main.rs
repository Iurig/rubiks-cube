use std::io::Stdout;

use ansi_to_tui::IntoText;
use anyhow::ensure;
use clap::Parser;
use crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    text::Line,
    widgets::{Block, Paragraph},
};
use rubiks_cube::{Cube3x3, Method, Puzzle, Roux, ZZ};

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

fn walkthrough_recon(index: usize, recon: &str, frame: &mut Frame) {
    let mut recon_iter = recon.lines();

    let scramble_text = recon_iter
        .next()
        .expect("reconstruction should have 1 scramble line");

    let solve_text = recon_iter.map(Line::from).collect::<Vec<_>>();

    let cube_widget = Paragraph::new(
        Cube3x3::from_moves(scramble_text)
            .unwrap_or_else(|_| panic!("scramble {scramble_text} doesn't parse"))
            .colored()
            .to_string()
            .as_bytes()
            .into_text()
            .unwrap(),
    )
    .block(Block::bordered().title(format!("Solve {}", index + 1)));

    let scramble_widget = Paragraph::new(scramble_text).block(Block::bordered().title("Scramble"));

    let solve_widget = Paragraph::new(solve_text).block(Block::bordered().title("Solve"));

    let help_widgets = [
        Line::from("Inspect solve: Enter").centered(),
        Line::from("Next solve: N").centered(),
        Line::from("Previous solve: P").centered(),
        Line::from("Exit: Esc").centered(),
    ];

    const CUBE_HEIGHT: u16 = 11;
    const CUBE_LENGTH: u16 = 13; // ignores back
    let layout = Layout::vertical([
        Constraint::Length(CUBE_HEIGHT),
        Constraint::Fill(1),
        Constraint::Length(2),
    ]);

    let [cube_space, recon_space, help_space] = layout.areas(frame.area());

    let [_, cube_area, _] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(CUBE_LENGTH),
        Constraint::Fill(1),
    ])
    .areas(cube_space);

    let [scramble_area, solve_area] =
        Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).areas(recon_space);

    let help_areas = Layout::horizontal((0..help_widgets.len()).fold(
        Vec::new(),
        |mut constraints, _| {
            constraints.push(Constraint::Fill(1));
            constraints
        },
    ))
    .split(help_space);

    frame.render_widget(cube_widget, cube_area);
    frame.render_widget(scramble_widget, scramble_area);
    frame.render_widget(solve_widget, solve_area);

    for i in 0..help_widgets.len() {
        frame.render_widget(help_widgets[i].clone(), help_areas[i]);
    }
}

fn inspect_solve(index: usize, recon: &str, terminal: &mut Terminal<CrosstermBackend<Stdout>>) {
    todo!(
        "inspect solve {index}, represented by {recon}, using {:?}",
        terminal
    )
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
                terminal.draw(|frame| {
                    walkthrough_recon(
                        index,
                        recon.get(index).unwrap_or_else(|| {
                            panic!(
                                "index should be between 0 and {}, but is {}",
                                recon.len() - 1,
                                index
                            )
                        }),
                        frame,
                    );
                })?;
                if let Some(KeyEvent {
                    code: pressed_key,
                    modifiers: KeyModifiers::NONE,
                    kind: _,
                    state: _,
                }) = event::read()?.as_key_press_event()
                {
                    match pressed_key {
                        KeyCode::Esc => break Ok(()),
                        KeyCode::Char('p') => {
                            index -= 1;
                            index = std::cmp::max(index, 0);
                        }
                        KeyCode::Char('n') => {
                            index += 1;
                            index = std::cmp::min(index, recon.len() - 1);
                        }
                        KeyCode::Enter => inspect_solve(
                            index,
                            recon.get(index).unwrap_or_else(|| {
                                panic!(
                                    "index should be between 0 and {}, but is {}",
                                    recon.len() - 1,
                                    index
                                )
                            }),
                            terminal,
                        ),
                        _ => (),
                    }
                }
            }
        })?;
    } else {
        for (i, solve) in recon.into_iter().enumerate() {
            println!("{}. {solve}", i + 1);
        }
    }
    Ok(())
}
