#[derive(Debug, PartialEq, Eq)]
pub enum Message {
    ExitProgram,
    Inspect(InspectMessage),
    BrowseSolves(BrowseMessage),
}

#[derive(Debug, PartialEq, Eq)]
pub enum InspectMessage {
    PreviousMove,
    NextMove,
    Exit,
}

#[derive(Debug, PartialEq, Eq)]
pub enum BrowseMessage {
    PreviousSolve,
    NextSolve,
    EnterInspect,
}
