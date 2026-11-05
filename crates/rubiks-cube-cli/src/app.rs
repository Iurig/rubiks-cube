use crate::message::{BrowseMessage, InspectMessage, Message};

#[derive(Debug)]
pub struct App {
    pub solve_index: usize,
    pub all_recons: Vec<String>,
    pub mode: Mode,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum Mode {
    Inspect {
        step: usize,
        move_in_step: usize,
    },
    #[default]
    BrowseSolves,
    Exited,
}

impl Mode {
    pub const fn is_inspect(&self) -> bool {
        matches!(
            self,
            Self::Inspect {
                step: _,
                move_in_step: _
            }
        )
    }
}

impl App {
    pub fn scramble(&self) -> &str {
        self.all_recons
            .get(self.solve_index)
            .unwrap_or_else(|| {
                panic!(
                    "index should be between 0 and {}, but is {}",
                    self.all_recons.len() - 1,
                    self.solve_index
                )
            })
            .lines()
            .next()
            .unwrap_or_else(|| panic!("all recons should have a scramble line"))
    }

    pub fn solution(&self) -> impl Iterator<Item = &str> {
        let mut view = self
            .all_recons
            .get(self.solve_index)
            .unwrap_or_else(|| {
                panic!(
                    "index should be between 0 and {}, but is {}",
                    self.all_recons.len() - 1,
                    self.solve_index
                )
            })
            .lines();
        view.next();
        view
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::BrowseSolves(message) => match message {
                BrowseMessage::PreviousSolve => {
                    self.solve_index = self.solve_index.saturating_sub(1);
                }

                BrowseMessage::NextSolve => {
                    self.solve_index =
                        std::cmp::min(self.solve_index + 1, self.all_recons.len() - 1);
                }
                BrowseMessage::EnterInspect => {
                    self.mode = Mode::Inspect {
                        step: 0,
                        move_in_step: 0,
                    };
                }
            },

            Message::Inspect(message) => match message {
                InspectMessage::PreviousMove => {
                    if self.mode.cur_move() == 0 {
                        if self.mode.cur_step() != 0 {
                            *self.mode.mut_step() -= 1;
                            *self.mode.mut_move() = total_moves_in_cur_step(self);
                        }
                    } else {
                        *self.mode.mut_move() -= 1;
                    }
                }

                InspectMessage::NextMove => {
                    let total_moves = total_moves_in_cur_step(self);

                    if self.mode.cur_move() + 1 > total_moves {
                        if self.mode.cur_step() <= self.solution().count() - 2 {
                            *self.mode.mut_move() = 0;
                            *self.mode.mut_step() += 1;
                        }
                    } else {
                        *self.mode.mut_move() += 1;
                    }
                }

                InspectMessage::PreviousStep => {
                    *self.mode.mut_step() = self.mode.cur_step().saturating_sub(1);

                    let total_moves = total_moves_in_cur_step(self);

                    *self.mode.mut_move() = std::cmp::min(self.mode.cur_move(), total_moves);
                }

                InspectMessage::NextStep => {
                    let total_steps = self.solution().count() - 1;

                    *self.mode.mut_step() = std::cmp::min(self.mode.cur_step() + 1, total_steps);

                    let total_moves = total_moves_in_cur_step(self);

                    *self.mode.mut_move() = std::cmp::min(self.mode.cur_move(), total_moves);
                }

                InspectMessage::Exit => self.mode = Mode::BrowseSolves,
            },

            Message::ExitProgram => self.mode = Mode::Exited,
        }
    }

    pub fn step_text(&self) -> &str {
        assert!(self.mode.is_inspect());
        self.solution()
            .nth(self.mode.cur_step())
            .expect("current step should have text")
    }
}

pub fn total_moves_in_cur_step(app: &App) -> usize {
    const EXTRA_SPACES: usize = 4;
    app.solution()
        .nth(app.mode.cur_step())
        .expect("current step doesn't exist")
        .chars()
        .take_while(|&c| c != '/')
        .filter(|&c| c == ' ')
        .count()
        - EXTRA_SPACES
}

impl Mode {
    pub fn mut_move(&mut self) -> &mut usize {
        if let Self::Inspect {
            step: _,
            move_in_step,
        } = self
        {
            move_in_step
        } else {
            panic!("not in inspect mode")
        }
    }

    pub fn mut_step(&mut self) -> &mut usize {
        if let Self::Inspect {
            step,
            move_in_step: _,
        } = self
        {
            step
        } else {
            panic!("not in inspect mode")
        }
    }

    pub fn cur_move(&self) -> usize {
        if let Self::Inspect {
            step: _,
            move_in_step,
        } = self
        {
            *move_in_step
        } else {
            panic!("not in inspect mode")
        }
    }

    pub fn cur_step(&self) -> usize {
        if let Self::Inspect {
            step,
            move_in_step: _,
        } = self
        {
            *step
        } else {
            panic!("not in inspect mode")
        }
    }
}
