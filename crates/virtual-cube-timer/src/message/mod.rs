use rubiks_cube::Move3x3;

pub enum Message {
    ExitProgram,
}

pub enum NavigationMessage {
    PreviousMove,
    NextMove,
    PreviousStep,
    NextStep,
    PreviousSolve,
    NextSolve,
    EnterSolves,
    EnterInspect,
}

pub enum CubeMessage {
    Apply(Move3x3),
}
