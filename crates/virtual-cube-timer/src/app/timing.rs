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
use rubiks_cube::{Algorithm, Cube3x3, Puzzle, Solution};

use crate::{
    event::Event,
    keybinds::{KeyPress, Keybinds},
    message::{CubeMessage, Message, TimingMessage},
    solves::{ReconWithScramble, SolveList, TimedSolve},
};

#[derive(Debug)]
pub struct TimingApp {
    pub cube: Cube3x3,
    pub method: crate::ValidMethods,
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
                    );
                    unlocked_solves.push(solve);
                });
            }
            TimingMessage::Next => match self.timer_state {
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

    pub fn update_with_message(&mut self, event: Event, keybinds: &Keybinds) -> Option<Message> {
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
                        self.update(&TimingMessage::Next);
                    }
                    _ if let Some(m) =
                        keybinds.as_move(&KeyPress::with_modifiers(code, modifiers)) =>
                    {
                        self.update(&TimingMessage::Cube(CubeMessage::Apply(m)));
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
