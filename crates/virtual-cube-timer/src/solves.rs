use std::{fmt::Display, fs, io::ErrorKind, str::FromStr, time::Duration};

use dirs::config_dir;
use rubiks_cube::{Algorithm, Cube3x3, Solution};
use serde_with::{DeserializeFromStr, SerializeDisplay};

#[derive(Debug, Clone, PartialEq, Eq, SerializeDisplay, DeserializeFromStr)]
pub struct SolveList(Vec<Solve>);

#[derive(Debug, Clone, PartialEq, Eq, SerializeDisplay, DeserializeFromStr)]
pub struct Solve {
    moves: SolveMoves,
    time: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolveMoves {
    scramble: Algorithm<Cube3x3>,
    solution: Solution<Cube3x3>,
}

impl Solve {
    pub const fn scramble(&self) -> &Algorithm<Cube3x3> {
        &self.moves.scramble
    }

    pub const fn solution(&self) -> &Solution<Cube3x3> {
        &self.moves.solution
    }
}

impl SolveMoves {
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
    const fn new() -> Self {
        Self(Vec::new())
    }

    pub fn from_solves_file() -> anyhow::Result<Self> {
        let dir = config_dir()
            .map(|dir| dir.join("virtual_cube_timer"))
            .expect("manually writen");

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
}

impl Display for Solve {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n\n{}", self.scramble(), self.solution())
    }
}

impl FromStr for SolveMoves {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            scramble: s
                .lines()
                .next()
                .ok_or(anyhow::format_err!("reconstruction has no first line"))?
                .parse()?,
            solution: s.lines().skip(1).collect::<String>().parse()?,
        })
    }
}

impl FromStr for Solve {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((time, solve)) = s.split_once(':') {
            Ok(Self {
                moves: solve.parse()?,
                time: parse_to_duration(time)?,
            })
        } else {
            Err(anyhow::format_err!("invalid string"))
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
                        seconds + multiplier * (text.parse::<u64>()?),
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
        Err(anyhow::format_err!("invalid string"))
    }
}

impl Display for SolveList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, solve) in self.0.iter().enumerate() {
            writeln!(f, "{}. {}\n\n", index + 1, solve)?;
        }
        Ok::<(), std::fmt::Error>(())
    }
}

impl FromStr for SolveList {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(
            s.split("\n\n")
                .map(str::parse::<Solve>)
                .collect::<Result<Vec<_>, _>>()?,
        ))
    }
}
