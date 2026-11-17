use rubiks_cube::Move3x3;

#[derive(Debug, PartialEq, Eq)]
pub enum Message {
    ExitProgram,
}

#[derive(Debug, PartialEq, Eq)]
pub enum NavigationMessage {
    EnterList,
    EnterSolve,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SolveNavMessage {
    PreviousMove,
    NextMove,
    PreviousStep,
    NextStep,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ListNavMessage {
    PreviousSolve,
    NextSolve,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CubeMessage {
    Apply(Move3x3),
}

#[derive(Debug, PartialEq, Eq)]
pub enum TimingMessage {
    Start,
    Stop,
    NextSolve,
    Cube(CubeMessage),
}
