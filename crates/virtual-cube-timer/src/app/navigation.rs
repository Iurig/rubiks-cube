use crossterm::event::{KeyCode, KeyEvent};
use rubiks_cube::{Algorithm, Cube3x3, Segment, Solution};

use crate::{
    message::{Message, NavigationMessage},
    solves::ReconWithScramble,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct NavigationApp {
    pub solve_index: usize,
    pub all_recons: Vec<ReconWithScramble>,
    pub nav_mode: NavigationMode,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NavigationMode {
    Solves,
    Inspect(InspectOptions),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct InspectOptions {
    pub move_index: usize,
    pub step_index: usize,
}

impl NavigationApp {
    pub fn update(&mut self, message: &NavigationMessage) {
        fn clamp_move(app: &mut NavigationApp) {
            let max_move_count = app.cur_step_move_count().expect("move count is invalid");
            if let Some(opt) = app.mut_inspect_options() {
                opt.move_index = std::cmp::min(opt.move_index, max_move_count);
            } else {
                panic!("cannot clamp move: solve not in inspect mode")
            }
        }

        match message {
            NavigationMessage::EnterInspect => {
                self.nav_mode = NavigationMode::Inspect(InspectOptions {
                    move_index: 0,
                    step_index: 0,
                });
            }
            NavigationMessage::EnterSolves => self.nav_mode = NavigationMode::Solves,
            NavigationMessage::PreviousMove => match self.nav_mode {
                NavigationMode::Solves => unreachable!(),
                NavigationMode::Inspect(InspectOptions {
                    move_index,
                    step_index,
                }) => {
                    self.nav_mode = NavigationMode::Inspect(InspectOptions {
                        move_index: move_index.saturating_sub(1),
                        step_index,
                    });
                }
            },
            NavigationMessage::NextMove => match self.nav_mode {
                NavigationMode::Solves => unreachable!(),
                NavigationMode::Inspect(InspectOptions {
                    move_index,
                    step_index,
                }) => {
                    self.nav_mode = NavigationMode::Inspect(InspectOptions {
                        move_index: std::cmp::min(
                            move_index + 1,
                            self.cur_step_move_count().expect("cur_step_move_count didn't return a valid move count for the current step"),
                        ),
                        step_index,
                    });
                }
            },
            NavigationMessage::PreviousStep => match self.nav_mode {
                NavigationMode::Solves => unreachable!(),
                NavigationMode::Inspect(InspectOptions {
                    move_index,
                    step_index,
                }) => {
                    self.nav_mode = NavigationMode::Inspect(InspectOptions {
                        move_index,
                        step_index: step_index.saturating_sub(1),
                    });
                    clamp_move(self);
                }
            },
            NavigationMessage::NextStep => match self.nav_mode {
                NavigationMode::Solves => unreachable!(),
                NavigationMode::Inspect(InspectOptions {
                    move_index,
                    step_index,
                }) => {
                    self.nav_mode = NavigationMode::Inspect(InspectOptions {
                        move_index,
                        step_index: std::cmp::min(step_index + 1, self.step_count() - 1),
                    });
                    clamp_move(self);
                }
            },
            NavigationMessage::PreviousSolve => todo!(),
            NavigationMessage::NextSolve => todo!(),
        }
    }
    pub fn update_with_message(&mut self, event: KeyEvent) -> Option<Message> {
        let KeyEvent {
            code: pressed_key,
            modifiers: _,
            kind: _,
            state: _,
        } = event;

        match self.nav_mode {
            NavigationMode::Solves => match pressed_key {
                KeyCode::Esc => return Some(Message::ExitProgram),

                KeyCode::Char('p' | 'P') => {
                    self.update(&NavigationMessage::PreviousSolve);
                }

                KeyCode::Char('n' | 'N') => {
                    self.update(&NavigationMessage::NextSolve);
                }

                KeyCode::Enter => {
                    self.update(&NavigationMessage::EnterInspect);
                }

                _ => (),
            },

            NavigationMode::Inspect(InspectOptions {
                move_index: _,
                step_index: _,
            }) => match pressed_key {
                KeyCode::Esc => self.update(&NavigationMessage::EnterSolves),

                KeyCode::Left => {
                    self.update(&NavigationMessage::PreviousMove);
                }

                KeyCode::Right => {
                    self.update(&NavigationMessage::NextMove);
                }

                KeyCode::Up => {
                    self.update(&NavigationMessage::PreviousStep);
                }

                KeyCode::Down => self.update(&NavigationMessage::NextStep),

                _ => (),
            },
        }
        None
    }

    pub const fn inspect_options(&self) -> Option<&InspectOptions> {
        match self.nav_mode {
            NavigationMode::Solves => None,
            NavigationMode::Inspect(ref options) => Some(options),
        }
    }

    pub const fn mut_inspect_options(&mut self) -> Option<&mut InspectOptions> {
        match self.nav_mode {
            NavigationMode::Solves => None,
            NavigationMode::Inspect(ref mut options) => Some(options),
        }
    }

    #[track_caller]
    pub const fn unchecked_inspect_options(&self) -> &InspectOptions {
        self.inspect_options().expect("not in inspect mode")
    }

    pub fn solution(&self) -> &Solution<Cube3x3> {
        self.all_recons
            .get(self.solve_index)
            .expect("index doesn't match an existing reconstruction")
            .solution()
    }

    pub fn scramble(&self) -> &Algorithm<Cube3x3> {
        self.all_recons
            .get(self.solve_index)
            .expect("index doesn't match an existing reconstruction")
            .scramble()
    }

    pub fn step_count(&self) -> usize {
        self.solution().iter().count()
    }

    pub fn step(&self) -> Option<&Segment<Cube3x3>> {
        Some(
            self.solution()
                .iter()
                .nth(self.inspect_options()?.step_index + 1)
                .expect("invalid step_index"),
        )
    }

    pub fn cur_step_move_count(&self) -> Option<usize> {
        Some(self.step().iter().count())
    }

    pub fn solve_iter(&self) -> impl Iterator<Item = Segment<Cube3x3>> {
        self.solution().iter().cloned()
    }
}
