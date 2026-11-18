use std::{
    collections::VecDeque,
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver},
    },
    thread,
    time::{Duration, Instant},
};

use crossterm::event::{KeyCode, KeyEvent};
use rubiks_cube::{Algorithm, Cube3x3, Puzzle, Solution, Technique};

use crate::{
    ValidMethods,
    event::Event,
    keybinds::{KeyPress, Keybinds},
    message::{CubeMessage, Message, TimingMessage},
    solves::{ReconWithScramble, SolveList, TimedSolve},
};

#[derive(Debug)]
pub struct TimingApp {
    pub cube: Cube3x3,
    pub technique: Technique<Cube3x3>,
    pub keybinds: Keybinds,
    pub step_number: usize,
    pub solves: Arc<Mutex<SolveList>>,
    pub curr_solve_scramble: VecDeque<Receiver<Algorithm<Cube3x3>>>,
    pub curr_solve_moves: Solution<Cube3x3>,
    pub starting_time: Instant,
    pub timer_number: Duration,
    pub timer_state: TimerState,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TimerState {
    Scrambled,
    Solving,
    Solved,
}

impl TimingApp {
    pub fn update(&mut self, message: &TimingMessage) {
        match message {
            TimingMessage::Cube(CubeMessage::Apply(m)) => self.cube *= *m,
            TimingMessage::Start => {
                self.timer_state = TimerState::Solving;
                self.starting_time = Instant::now();
            }
            TimingMessage::Stop => {
                self.timer_state = TimerState::Solved;
                self.timer_number = self.starting_time.elapsed();
                let scramble_receiver = self
                    .curr_solve_scramble
                    .pop_front()
                    .expect("no receivers from solves");
                let solves_to_send = self.solves.clone();
                let solve_moves = self.curr_solve_moves.clone();
                let solve_time = self.timer_number;
                thread::spawn(move || {
                    let mut unlocked_solves = solves_to_send.lock().expect("mutex should be clean");
                    let solve = TimedSolve::new(
                        ReconWithScramble::new(
                            scramble_receiver.recv().expect("failed to save solve"),
                            solve_moves,
                        ),
                        solve_time,
                        ValidMethods::Roux,
                    );
                    unlocked_solves.push(solve);
                });
            }
            TimingMessage::NextSolve => match self.timer_state {
                TimerState::Scrambled => self.update(&TimingMessage::Start),
                TimerState::Solving => self.update(&TimingMessage::Stop),
                TimerState::Solved => {
                    self.timer_state = TimerState::Scrambled;
                    self.cube = Cube3x3::scrambled();
                    let (sender, receiver) = mpsc::channel::<Algorithm<Cube3x3>>();
                    let cube_state = self.cube;
                    self.curr_solve_scramble.push_back(receiver);
                    thread::spawn(move || {
                        sender.send(cube_state.get_scramble().expect("cube should solve"))
                    });
                    self.timer_number = Duration::ZERO;
                }
            },
        }
    }

    pub fn update_with_message(&mut self, event: Event) -> Option<Message> {
        match event {
            Event::Key(event) => {
                let KeyEvent {
                    code,
                    modifiers,
                    kind: _,
                    state: _,
                } = event;

                match (code, modifiers) {
                    (KeyCode::Esc, _) => return Some(Message::ExitProgram),
                    (KeyCode::Char(' '), _) => {
                        self.update(&TimingMessage::NextSolve);
                    }
                    _ if let Some(m) = self
                        .keybinds
                        .as_move(&KeyPress::with_modifiers(code, modifiers)) =>
                    {
                        while self
                            .technique
                            .steps()
                            .nth(self.step_number)
                            .is_some_and(|s| s.is_done(&self.cube))
                        {
                            self.step_number += 1;
                        }

                        self.update(&TimingMessage::Cube(CubeMessage::Apply(m)));

                        self.curr_solve_moves[self.step_number].push(m);

                        if self.timer_state == TimerState::Scrambled {
                            self.update(&TimingMessage::Start);
                        }
                        if self.cube.is_solved() {
                            self.update(&TimingMessage::Stop);
                        }
                    }
                    _ => (),
                }
            }
            Event::Tick => {
                if self.timer_state == TimerState::Solving {
                    self.timer_number = self.starting_time.elapsed();
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {

    use rubiks_cube::{Method, Move3x3, MoveModifier, Roux, SolutionSegment, Step};

    use crate::keybinds::key_press;

    use super::*;

    pub fn move_to_key(binds: &Keybinds, m: <Cube3x3 as Puzzle>::Move) -> Option<KeyPress> {
        binds.moves.get_by_right(&m).copied()
    }

    #[test]
    fn solution_is_constructed_with_messages() {
        let scrambled_cube =
            Cube3x3::from_moves("D' F' U2 D' R' F' B R B2 D' B2 D2 B2 R2 B2 U' R2 F2 B2 L")
                .unwrap();

        let (sender, receiver) = mpsc::channel();

        sender
            .send(Algorithm::from_static(
                "D' F' U2 D' R' F' B R B2 D' B2 D2 B2 R2 B2 U' R2 F2 B2 L",
            ))
            .unwrap();

        let mut app = TimingApp {
            cube: scrambled_cube,
            technique: Roux::default().to_technique(),
            keybinds: Keybinds::default(),
            step_number: 0,
            solves: Arc::new(Mutex::new(SolveList::new())),
            curr_solve_scramble: VecDeque::from([receiver]),
            curr_solve_moves: Solution::with_empty_steps(
                Roux::default().to_technique().steps().map(Step::name),
            ),
            starting_time: Instant::now(),
            timer_number: Duration::ZERO,
            timer_state: TimerState::Scrambled,
        };

        let cube_recon = Roux::default()
            .to_technique()
            .solve(&mut scrambled_cube.clone())
            .unwrap();

        let expanded_cube_recon = cube_recon
            .iter()
            .map(|s| {
                SolutionSegment::with_moves(
                    s.name(),
                    s.iter()
                        .flat_map(|m| {
                            if m.modifier() == MoveModifier::Double {
                                let inner_move = Move3x3::new(m.part(), MoveModifier::Clockwise);
                                vec![inner_move, inner_move].into_iter()
                            } else {
                                vec![*m].into_iter()
                            }
                        })
                        .collect::<Algorithm<Cube3x3>>(),
                )
            })
            .collect::<Solution<Cube3x3>>();

        for m in expanded_cube_recon.moves() {
            let key = move_to_key(&Keybinds::default(), *m).unwrap_or_else(|| panic!("{m}"));
            app.update_with_message(Event::Key(KeyEvent::new(key.code(), key.modifiers())));
        }

        assert!(app.cube.is_solved());
        assert_eq!(expanded_cube_recon, app.curr_solve_moves);

        app.update_with_message(Event::Key(key_press!(Char(' '))));

        assert_eq!(app.timer_state, TimerState::Scrambled);

        assert_eq!(
            app.update_with_message(Event::Key(key_press!(Esc))),
            Some(Message::ExitProgram)
        );
    }
}
