use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent};
use rubiks_cube::{Cube3x3, Puzzle, Solution};

use crate::{
    event::Event,
    keybinds::{KeyPress, Keybinds},
    message::{CubeMessage, Message, TimingMessage},
    solves::SolveList,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TimingApp {
    pub cube: Cube3x3,
    pub method: crate::ValidMethods,
    pub solves: SolveList,
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
            }
            TimingMessage::Next => match self.timer_state {
                TimerState::Scrambled => self.update(&TimingMessage::Start),
                TimerState::Solving => self.update(&TimingMessage::Stop),
                TimerState::Solved => {
                    self.timer_state = TimerState::Scrambled;
                    self.cube = Cube3x3::scrambled();
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
