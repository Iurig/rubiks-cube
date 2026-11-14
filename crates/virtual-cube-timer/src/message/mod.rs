use rubiks_cube::Move3x3;

pub enum Message {
    ExitProgram,
}

pub enum NavigationMessage {
    EnterList,
    EnterSolve,
}

pub enum SolveNavMessage {
    PreviousMove,
    NextMove,
    PreviousStep,
    NextStep,
}

pub enum ListNavMessage {
    PreviousSolve,
    NextSolve,
}

pub enum CubeMessage {
    Apply(Move3x3),
}

pub enum TimingMessage {
    Start,
    Stop,
    Next,
    Cube(CubeMessage),
}
