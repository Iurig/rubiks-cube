use rubiks_cube::Cube3x3;

#[derive(Debug)]
pub struct App {
    pub mode: Mode,
    pub cube: Cube3x3,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Mode {
    Untimed,
    Timed,
    Exited,
}
