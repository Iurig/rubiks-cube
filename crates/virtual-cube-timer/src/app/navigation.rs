use crossterm::event::{KeyCode, KeyEvent};
use rubiks_cube::{Algorithm, Cube3x3, Segment, Solution};

use crate::{
    message::{ListNavMessage, Message, NavigationMessage, SolveNavMessage},
    solves::ReconWithScramble,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SolveNavApp {
    pub move_index: usize,
    pub step_index: usize,
    pub list_app: ListNavApp,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ListNavApp {
    pub solve_index: usize,
    pub all_recons: Vec<ReconWithScramble>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NavigationApp {
    List(ListNavApp),
    Solve(SolveNavApp),
}

impl ListNavApp {
    pub fn update(&mut self, message: &ListNavMessage) {
        match message {
            ListNavMessage::PreviousSolve => self.solve_index = self.solve_index.saturating_sub(1),
            ListNavMessage::NextSolve => {
                self.solve_index = std::cmp::min(self.solve_index + 1, self.all_recons.len());
            }
        }
    }
}

impl SolveNavApp {
    pub fn update(&mut self, message: &SolveNavMessage) {
        fn clamp_move(app: &mut SolveNavApp) {
            let max_move_count = app.cur_step_move_count();
            app.move_index = std::cmp::min(app.move_index, max_move_count);
        }
        match message {
            SolveNavMessage::PreviousMove => {
                self.move_index = self.move_index.saturating_sub(1);
            }
            SolveNavMessage::NextMove => {
                self.move_index = std::cmp::min(self.move_index + 1, self.cur_step_move_count());
            }
            SolveNavMessage::PreviousStep => {
                self.step_index = self.step_index.saturating_sub(1);
                clamp_move(self);
            }
            SolveNavMessage::NextStep => {
                self.step_index =
                    std::cmp::min(self.step_index + 1, self.list_app.step_count() - 1);
                clamp_move(self);
            }
        }
    }

    pub fn step(&self) -> Option<&Segment<Cube3x3>> {
        self.list_app.solution().iter().nth(self.step_index + 1)
    }

    pub fn cur_step_move_count(&self) -> usize {
        self.step().iter().count()
    }
}

impl ListNavApp {
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

    pub fn solve_iter(&self) -> impl Iterator<Item = Segment<Cube3x3>> {
        self.solution().iter().cloned()
    }
}

impl NavigationApp {
    pub fn update(&mut self, message: &NavigationMessage) {
        match message {
            NavigationMessage::EnterSolve => {
                *self = match self {
                    Self::List(list_app) => Self::Solve(SolveNavApp {
                        move_index: 0,
                        step_index: 0,
                        list_app: list_app.clone(),
                    }),
                    Self::Solve(_) => unreachable!(),
                }
            }
            NavigationMessage::EnterList => {
                *self = match self {
                    Self::Solve(solve_app) => Self::List(ListNavApp {
                        solve_index: solve_app.list_app.solve_index,
                        all_recons: solve_app.list_app.all_recons.clone(),
                    }),
                    Self::List(_) => unreachable!(),
                }
            }
        }
    }
    pub fn update_with_message(&mut self, event: KeyEvent) -> Option<Message> {
        let KeyEvent {
            code: pressed_key,
            modifiers: _,
            kind: _,
            state: _,
        } = event;

        match pressed_key {
            KeyCode::Esc => match self {
                Self::List(_) => return Some(Message::ExitProgram),
                Self::Solve(_) => self.update(&NavigationMessage::EnterList),
            },

            KeyCode::Char('p' | 'P') => self.mut_list_app().update(&ListNavMessage::PreviousSolve),

            KeyCode::Char('n' | 'N') => self.mut_list_app().update(&ListNavMessage::NextSolve),

            pressed_key => match self {
                Self::Solve(solve_app) => match pressed_key {
                    KeyCode::Left => solve_app.update(&SolveNavMessage::PreviousMove),
                    KeyCode::Right => solve_app.update(&SolveNavMessage::NextMove),
                    KeyCode::Up => solve_app.update(&SolveNavMessage::PreviousStep),
                    KeyCode::Down => solve_app.update(&SolveNavMessage::NextStep),
                    _ => (),
                },
                Self::List(_) => {
                    if pressed_key == KeyCode::Enter {
                        self.update(&NavigationMessage::EnterSolve);
                    }
                }
            },
        }
        None
    }

    pub const fn mut_list_app(&mut self) -> &mut ListNavApp {
        match self {
            Self::List(list_app) => list_app,
            Self::Solve(solve_app) => &mut solve_app.list_app,
        }
    }

    pub const fn list_app(&self) -> &ListNavApp {
        match self {
            Self::List(list_app) => list_app,
            Self::Solve(solve_app) => &solve_app.list_app,
        }
    }
}
