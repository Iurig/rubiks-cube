use std::{fmt::Display, fs, io::ErrorKind, str::FromStr, time::Duration};

use anyhow::anyhow;
use dirs::config_dir;
use rubiks_cube::{Algorithm, Cube3x3, ParseSequenceError, Segment, Solution};
use serde_with::{DeserializeFromStr, SerializeDisplay};

#[derive(Debug, Clone, PartialEq, Eq, SerializeDisplay, DeserializeFromStr)]
pub struct SolveList(Vec<(usize, TimedSolve)>);

#[derive(Debug, Clone, PartialEq, Eq, SerializeDisplay, DeserializeFromStr)]
pub struct TimedSolve {
    moves: ReconWithScramble,
    time: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconWithScramble {
    scramble: Algorithm<Cube3x3>,
    solution: Solution<Cube3x3>,
}

impl TimedSolve {
    pub const fn new(moves: ReconWithScramble, time: Duration) -> Self {
        Self { moves, time }
    }

    pub const fn scramble(&self) -> &Algorithm<Cube3x3> {
        &self.moves.scramble
    }

    pub const fn solution(&self) -> &Solution<Cube3x3> {
        &self.moves.solution
    }

    const fn time(&self) -> &Duration {
        &self.time
    }
}

impl ReconWithScramble {
    pub const fn new(scramble: Algorithm<Cube3x3>, solution: Solution<Cube3x3>) -> Self {
        Self { scramble, solution }
    }

    pub const fn scramble(&self) -> &Algorithm<Cube3x3> {
        &self.scramble
    }

    pub const fn solution(&self) -> &Solution<Cube3x3> {
        &self.solution
    }
}

impl SolveList {
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

    pub fn from_solves_file() -> anyhow::Result<Self> {
        let dir = config_dir()
            .map(|dir| dir.join("virtual_cube_timer"))
            .expect("manually written");

        fs::create_dir_all(dir.clone())?;

        let solves_path = dir.join("solves.txt");

        match std::fs::read_to_string(&solves_path) {
            Ok(text) => text.parse::<Self>(),
            Err(e) if e.kind() == ErrorKind::NotFound => {
                fs::write(solves_path, Self::new().to_string())?;
                Ok(Self::new())
            }
            Err(e) => Err(e.into()),
        }
    }

    pub fn push(&mut self, solve: TimedSolve) {
        self.0.push((self.0.len(), solve));
    }
}

impl Display for TimedSolve {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}.{:.0?}: {}\n{}",
            self.time().as_secs(),
            self.time().as_millis(),
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
                .map(str::parse::<Segment<Cube3x3>>)
                .collect::<Result<Vec<_>, ParseSequenceError>>()?
                .into_iter()
                .collect(),
        })
    }
}

impl FromStr for TimedSolve {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((time, solve)) = s.split_once(':') {
            Ok(Self {
                moves: solve.parse()?,
                time: parse_to_duration(time)?,
            })
        } else {
            Err(anyhow::format_err!("invalid solve: {s}"))
        }
    }
}

pub fn parse_to_duration(text: &str) -> anyhow::Result<Duration> {
    if let Some((seconds_or_more, sub_second)) = text.split_once('.') {
        let secs = Duration::from_secs(
            seconds_or_more
                .split(':')
                .try_fold((0, 1), |(seconds, multiplier), text| {
                    anyhow::Ok((
                        seconds + multiplier * (text.trim().parse::<u64>()?),
                        multiplier * 60,
                    ))
                })?
                .0,
        );

        let milis = Duration::from_millis(
            (sub_second.parse::<u64>()? * 1000) / 10u64.pow(u32::try_from(sub_second.len())?),
        );

        Ok(secs + milis)
    } else {
        Err(anyhow::format_err!("invalid duration: {text}"))
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
    use rubiks_cube::{Method, Puzzle, Roux};

    use super::*;

    #[test]
    fn roux_solve_round_trip() -> anyhow::Result<()> {
        let solves = (0..10)
            .map(|_| -> anyhow::Result<TimedSolve> {
                let scramble = Cube3x3::scramble()?;
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
                    Duration::ZERO,
                ))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;
        let list = SolveList::from_solves(solves);

        dbg!(list.to_string());

        assert_eq!(
            list,
            SolveList::from_str(&list.to_string())
                .with_context(|| format!("{:?}", &list.to_string()))?
        );
        Ok(())
    }
}
