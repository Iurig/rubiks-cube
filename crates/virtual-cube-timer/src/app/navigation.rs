use crate::message::NavigationMessage;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct NavigationApp {
    pub solve_index: usize,
    pub demo: Option<DemoOptions>,
    pub all_recons: Vec<String>,
    pub nav_mode: NavigationMode,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DemoOptions {
    pub method: crate::Method,
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
    pub fn inspect_options(&self) -> Option<&InspectOptions> {
        match self.nav_mode {
            NavigationMode::Solves => None,
            NavigationMode::Inspect(ref options) => Some(options),
        }
    }

    pub fn mut_inspect_options(&mut self) -> Option<&mut InspectOptions> {
        match self.nav_mode {
            NavigationMode::Solves => None,
            NavigationMode::Inspect(ref mut options) => Some(options),
        }
    }

    #[track_caller]
    pub fn unchecked_inspect_options(&self) -> &InspectOptions {
        self.inspect_options().expect("not in inspect mode")
    }

    pub fn recon(&self) -> &str {
        self.all_recons
            .get(self.solve_index)
            .expect("index doesn't match an existing reconstruction")
    }

    pub fn scramble(&self) -> &str {
        self.recon()
            .lines()
            .next()
            .expect("solve should have a scramble line")
    }

    pub fn step_count(&self) -> usize {
        self.recon().lines().count() - 1
    }

    pub fn step(&self) -> Option<&str> {
        Some(
            &self
                .recon()
                .lines()
                .nth(self.inspect_options()?.step_index + 1)
                .expect("invalid step_index")
                .split_once('/')
                .expect("step didn't have comment")
                .0,
        )
    }

    pub fn cur_step_move_count(&self) -> Option<usize> {
        Some(self.step()?.chars().filter(|c| *c == ' ').count() - 4)
    }

    pub fn solve_iter(&self) -> impl Iterator<Item = &str> {
        self.recon().lines().skip(1)
    }

    pub fn navigate(&mut self, message: &NavigationMessage) {
        fn clamp_move(app: &mut NavigationApp) {
            let max_move_count = app.cur_step_move_count().unwrap();
            if let Some(opt) = app.mut_inspect_options() {
                opt.move_index = std::cmp::min(opt.move_index, max_move_count)
            } else {
                panic!("cannot clamp move: solve not in inspect mode")
            }
        }

        match message {
            NavigationMessage::EnterInspect => {
                self.nav_mode = NavigationMode::Inspect(InspectOptions {
                    move_index: 0,
                    step_index: 0,
                })
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
                    })
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
                            self.cur_step_move_count().unwrap(),
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
                    clamp_move(self)
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
}
