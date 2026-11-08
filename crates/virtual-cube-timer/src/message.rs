use rubiks_cube::Move3x3;

pub enum Message {
    ApplyMove(Move3x3),
    ExitProgram,
}
