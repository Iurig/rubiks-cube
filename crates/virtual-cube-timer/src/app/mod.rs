pub mod navigation;
pub mod timing;
pub mod untimed;

use crate::app::{navigation::NavigationApp, timing::TimingApp, untimed::UntimedApp};

#[derive(Debug, Clone)]
pub struct App {
    pub mode: Mode,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Mode {
    Untimed(UntimedApp),
    Timing(TimingApp),
    Navigation(NavigationApp),
}
