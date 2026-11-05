use crate::message::{BrowseMessage, InspectMessage, Message};

#[derive(Debug)]
pub struct App {
    pub solve_index: usize,
    pub all_recons: Vec<String>,
    pub mode: Mode,
    pub mode_info: ModeInfo,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum Mode {
    Inspect,
    #[default]
    BrowseSolves,
    Exited,
}

#[derive(Debug)]
pub enum ModeInfo {
    Inspect { step: usize, move_in_step: usize },
    BrowseSolves,
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
                    self.solve_index = self.solve_index.saturating_sub(1)
                }
                BrowseMessage::NextSolve => {
                    self.solve_index =
                        std::cmp::min(self.solve_index + 1, self.all_recons.len() - 1)
                }
                BrowseMessage::EnterInspect => {
                    self.mode = Mode::Inspect;
                    self.mode_info = ModeInfo::Inspect {
                        step: 0,
                        move_in_step: 0,
                    };
                }
            },

            Message::Inspect(message) => match message {
                InspectMessage::PreviousMove => *self.mode_info.mut_move() -= 1,
                InspectMessage::NextMove => *self.mode_info.mut_move() += 1,
                InspectMessage::Exit => self.mode = Mode::BrowseSolves,
            },

            Message::ExitProgram => self.mode = Mode::Exited,
        }
    }
}

impl ModeInfo {
    pub fn mut_move(&mut self) -> &mut usize {
        if let ModeInfo::Inspect {
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
        if let ModeInfo::Inspect {
            step,
            move_in_step: _,
        } = self
        {
            step
        } else {
            panic!("not in inspect mode")
        }
    }
}
