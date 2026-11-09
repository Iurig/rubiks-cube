use rubiks_cube::Cube3x3;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TimingApp {
    pub cube: Cube3x3,
    pub method: crate::Method,
}
