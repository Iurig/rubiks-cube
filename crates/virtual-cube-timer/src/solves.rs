use std::{fmt::Display, fs, io::ErrorKind, path::PathBuf, str::FromStr, time::Duration};

use anyhow::anyhow;
use dirs::config_dir;
use rubiks_cube::{Algorithm, Cube3x3, ParseSequenceError, Solution, SolutionSegment};
use serde_with::{DeserializeFromStr, SerializeDisplay};

use crate::{
    ValidMethods,
    time::{to_duration, to_text},
};

#[derive(Debug, Clone, PartialEq, Eq, SerializeDisplay, DeserializeFromStr)]
pub struct SolveList(Vec<(usize, TimedSolve)>);

#[derive(Debug, Clone, PartialEq, Eq, SerializeDisplay, DeserializeFromStr)]
pub struct TimedSolve {
    moves: ReconWithScramble,
    time: Duration,
    method: ValidMethods,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconWithScramble {
    scramble: Algorithm<Cube3x3>,
    solution: Solution<Cube3x3>,
}

impl TimedSolve {
    #[must_use]
    pub const fn new(moves: ReconWithScramble, time: Duration, method: ValidMethods) -> Self {
        Self {
            moves,
            time,
            method,
        }
    }

    #[must_use]
    pub const fn scramble(&self) -> &Algorithm<Cube3x3> {
        self.moves.scramble()
    }

    #[must_use]
    pub const fn solution(&self) -> &Solution<Cube3x3> {
        self.moves.solution()
    }

    #[must_use]
    pub const fn time(&self) -> &Duration {
        &self.time
    }

    #[must_use]
    pub const fn method(&self) -> &ValidMethods {
        &self.method
    }
}

impl ReconWithScramble {
    #[must_use]
    pub const fn new(scramble: Algorithm<Cube3x3>, solution: Solution<Cube3x3>) -> Self {
        Self { scramble, solution }
    }

    #[must_use]
    pub const fn scramble(&self) -> &Algorithm<Cube3x3> {
        &self.scramble
    }

    #[must_use]
    pub const fn solution(&self) -> &Solution<Cube3x3> {
        &self.solution
    }
}

impl SolveList {
    #[must_use]
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    pub fn from_solves(solves: impl IntoIterator<Item = TimedSolve>) -> Self {
        Self(
            solves
                .into_iter()
                .enumerate()
                .map(|(index, value)| (index + 1, value))
                .collect(),
        )
    }

    pub fn solves_path() -> anyhow::Result<PathBuf> {
        let dir = config_dir()
            .map(|dir| dir.join("virtual_cube_timer"))
            .expect("manually written");

        fs::create_dir_all(dir.clone())?;

        Ok(dir.join("solves.txt"))
    }

    pub fn from_solves_file(path: PathBuf) -> anyhow::Result<Self> {
        match std::fs::read_to_string(&path) {
            Ok(text) => text.parse::<Self>(),
            Err(e) if e.kind() == ErrorKind::NotFound => {
                fs::write(path, Self::new().to_string())?;
                Ok(Self::new())
            }
            Err(e) => Err(e.into()),
        }
    }

    pub fn push(&mut self, solve: TimedSolve) {
        self.0.push((self.0.len(), solve));
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    #[must_use]
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn get(&self, index: usize) -> Option<&TimedSolve> {
        self.0.get(index).map(|s| &s.1)
    }
}

impl Default for SolveList {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for TimedSolve {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {}\n{}",
            to_text(*self.time()),
            self.scramble(),
            self.solution()
        )
    }
}

impl FromStr for ReconWithScramble {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            scramble: s
                .lines()
                .next()
                .ok_or_else(|| anyhow::format_err!("reconstruction has no first line"))?
                .parse()?,
            solution: s
                .lines()
                .skip(1)
                .map(str::parse::<SolutionSegment<Cube3x3>>)
                .collect::<Result<Vec<_>, ParseSequenceError>>()?
                .into_iter()
                .collect(),
        })
    }
}

impl FromStr for TimedSolve {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((time, solve)) = s.rsplit_once(':') {
            Ok(Self::new(
                solve.parse()?,
                to_duration(time)?,
                ValidMethods::Roux,
            ))
        } else {
            Err(anyhow::format_err!("invalid solve: {s}"))
        }
    }
}

impl Display for SolveList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, solve) in &self.0 {
            writeln!(f, "{index}. {solve}")?;
        }
        Ok::<(), std::fmt::Error>(())
    }
}

impl FromStr for SolveList {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(
            s.split("\n\n")
                .flat_map(|s| {
                    let (number, solve) = s
                        .split_once('.')
                        .ok_or_else(|| anyhow!("didn't have .: {s}"))?;
                    Ok::<_, anyhow::Error>((number.parse::<usize>()?, solve.parse()?))
                })
                .collect::<Vec<_>>(),
        ))
    }
}

#[cfg(test)]
mod tests {

    use anyhow::{Context, Ok};
    use rubiks_cube::{Method, Roux};

    use super::*;

    #[test]
    fn roux_solve_round_trip() -> anyhow::Result<()> {
        let scrambles = [
            "D2 U L2 U2 R2 F2 U L2 D' F2 R2 U2 B' D' F U2 L2 D' L' B'",
            "B U F R B' D' F' D' L' B2 L2 F2 D2 R2 D2 R2 F L2 F2 L2",
            "U2 B2 D2 B2 L' R2 U2 R' B2 L' U2 D F' R2 B2 R D U R' F",
            "D2 F D R2 D2 F2 R' F2 R2 B2 U2 R' D2 L F' R' B R2 U2 L U'",
            "D' F L U' R D2 F2 L' F' U B2 U' L2 D2 B2 D' R2 U F2 L2",
            "R L2 D2 L' D2 F' U' F2 D F2 R' U2 L2 F2 B2 L B2 R F2 L",
            "L D2 L2 U2 L U2 L' D2 U2 L U2 B2 U B L2 F L R' D U R",
            "B D' L' F2 R F2 R U2 L D2 L F2 R U2 D' L2 F' D' L' U2 L2",
            "L' F2 R2 B2 U' R2 D' L2 F2 R2 U2 B2 F2 L B' L2 D' B' D2 R' B'",
            "L2 U2 R2 B2 U' B2 U2 R2 U L2 F2 L2 B' L U2 B' R' D' R' B",
        ];
        let solves = (0..10)
            .map(|i| -> anyhow::Result<TimedSolve> {
                let scramble = Algorithm::from_static(scrambles[i]);
                Ok(TimedSolve::new(
                    ReconWithScramble::new(
                        scramble.clone(),
                        Roux::new()
                            .solve(
                                &mut Cube3x3::from_moves(&scramble.to_string())
                                    .with_context(|| format!("{scramble:?}"))?,
                            )
                            .with_context(|| format!("{scramble:?}"))?,
                    ),
                    Duration::from_millis(fastrand::u64(0..2 * 3_600_000)),
                    ValidMethods::Roux,
                ))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;
        let list = SolveList::from_solves(solves);

        dbg!(list.to_string());

        assert_eq!(
            list,
            SolveList::from_str(&list.to_string())
                .with_context(|| format!("{:?}", list.to_string()))?
        );
        Ok(())
    }
}
