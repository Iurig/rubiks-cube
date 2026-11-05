use std::io::Stdout;

use ansi_to_tui::IntoText;
use anyhow::ensure;
use clap::{Parser, ValueEnum};
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

fn walkthrough_recon(index: usize, recon: &str, frame: &mut Frame) {
    const CUBE_HEIGHT: u16 = 11;
    const CUBE_LENGTH: u16 = 13; // ignores back

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
            .unwrap_or_else(|e| {
                panic!("original string should've been possible to parse to ratatui::Text: {e}")
            }),
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

    let help_areas =
        Layout::horizontal(help_widgets.iter().map(|_| Constraint::Fill(1))).split(help_space);

    frame.render_widget(cube_widget, cube_area);
    frame.render_widget(scramble_widget, scramble_area);
    frame.render_widget(solve_widget, solve_area);

    for (widget, area) in help_widgets.iter().zip(help_areas.iter()) {
        frame.render_widget(widget, *area);
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

    let recon = match args.method {
        Methods::Roux => reconstruct_n(args.number, &Roux::default())?,
        Methods::Zz => reconstruct_n(args.number, &ZZ::default())?,
    };

    if args.iterative {
        let mut index = 0;
        ratatui::run(|terminal| -> anyhow::Result<()> {
            loop {
                let current_recon = recon.get(index).unwrap_or_else(|| {
                    panic!(
                        "index should be between 0 and {}, but is {}",
                        recon.len() - 1,
                        index
                    )
                });
                terminal.draw(|frame| walkthrough_recon(index, current_recon, frame))?;
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
                            index = index.saturating_sub(1);
                        }
                        KeyCode::Char('n') => {
                            index += 1;
                            index = std::cmp::min(index, recon.len() - 1);
                        }
                        KeyCode::Enter => inspect_solve(index, current_recon, terminal),
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
