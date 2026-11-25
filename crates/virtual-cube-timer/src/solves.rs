use std::{fmt::Display, fs, io::ErrorKind, path::PathBuf, str::FromStr, time::Duration};

use anyhow::{Result, anyhow};
use dirs::config_dir;
use rubiks_cube::{
    Algorithm, Cube3x3, MethodSolution, ParseSequenceError, Reconstruction, StepSolution,
};
use serde_with::{DeserializeFromStr, SerializeDisplay};

use crate::{
    ValidMethods,
    time::{to_duration, to_text},
};

#[derive(Debug, Clone, PartialEq, Eq, SerializeDisplay, DeserializeFromStr)]
pub struct SolveList(Vec<(usize, TimedSolve)>);

#[derive(Debug, Clone, PartialEq, Eq, SerializeDisplay, DeserializeFromStr)]
pub struct TimedSolve {
    recon: Reconstruction<Cube3x3, ValidMethods>,
    time: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconWithScramble {
    scramble: Algorithm<Cube3x3>,
    solution: MethodSolution<Cube3x3>,
}

impl From<Reconstruction<Cube3x3, ValidMethods>> for ReconWithScramble {
    fn from(value: Reconstruction<Cube3x3, ValidMethods>) -> Self {
        Self {
            scramble: value.scramble().clone(),
            solution: value.solution().clone(),
        }
    }
}

impl TimedSolve {
    #[must_use]
    pub const fn new(recon: Reconstruction<Cube3x3, ValidMethods>, time: Duration) -> Self {
        Self { recon, time }
    }

    #[must_use]
    pub const fn scramble(&self) -> &Algorithm<Cube3x3> {
        self.recon.scramble()
    }

    #[must_use]
    pub const fn solution(&self) -> &MethodSolution<Cube3x3> {
        self.recon.solution()
    }

    #[must_use]
    pub const fn time(&self) -> &Duration {
        &self.time
    }

    #[must_use]
    pub const fn method(&self) -> &ValidMethods {
        self.recon.method()
    }
}

impl ReconWithScramble {
    #[must_use]
    pub const fn new(scramble: Algorithm<Cube3x3>, solution: MethodSolution<Cube3x3>) -> Self {
        Self { scramble, solution }
    }

    #[must_use]
    pub const fn scramble(&self) -> &Algorithm<Cube3x3> {
        &self.scramble
    }

    #[must_use]
    pub const fn solution(&self) -> &MethodSolution<Cube3x3> {
        &self.solution
    }
}

impl SolveList {
    #[must_use]
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    pub fn from_solves(solves: impl IntoIterator<Item = TimedSolve>) -> Self {
        Self(solves.into_iter().enumerate().collect())
    }

    pub fn solves_path() -> anyhow::Result<PathBuf> {
        let dir = config_dir()
            .map(|dir| dir.join("virtual_cube_timer"))
            .expect("manually written");

        fs::create_dir_all(dir.clone())?;

        Ok(dir.join("solves.txt"))
    }

    pub fn from_solves_file(path: &PathBuf) -> anyhow::Result<Self> {
        match std::fs::read_to_string(path) {
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

    pub fn extend(&mut self, other: impl IntoIterator<Item = TimedSolve>) {
        for solve in other {
            self.push(solve);
        }
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
            "{}({}): {}\n{}",
            to_text(*self.time()),
            self.method(),
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
                .map(str::parse::<StepSolution<Cube3x3>>)
                .collect::<Result<Vec<_>, ParseSequenceError>>()?
                .into_iter()
                .collect(),
        })
    }
}

impl FromStr for TimedSolve {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(((time, Some(method)), solve)) = s.rsplit_once(':').and_then(|(pre, post)| {
            pre.rsplit_once('(')
                .map(|(s1, s2)| ((s1, s2.strip_suffix(')')), post))
        }) {
            let mut lines = solve.lines();
            let mut recon = Reconstruction::new(
                lines
                    .next()
                    .ok_or_else(|| anyhow!("no first line"))?
                    .parse()?,
                method.parse()?,
            );
            recon.extend(
                lines
                    .map(|l| Ok(l.parse::<Algorithm<Cube3x3>>()?.into_iter()))
                    .collect::<Result<Vec<_>>>()?
                    .into_iter()
                    .flat_map(IntoIterator::into_iter),
            );
            Ok(Self::new(recon, to_duration(time)?))
        } else {
            Err(anyhow::format_err!("invalid solve: {s}"))
        }
    }
}

impl Display for SolveList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, solve) in &self.0 {
            writeln!(f, "{}. {solve}", index + 1)?;
        }
        Ok::<(), std::fmt::Error>(())
    }
}

impl FromStr for SolveList {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(
            s.split_terminator("\n\n")
                .flat_map(|s| {
                    let (number, solve) = s
                        .split_once('.')
                        .ok_or_else(|| anyhow!("didn't have .: {s}"))?;
                    Ok::<_, anyhow::Error>((number.parse::<usize>()? - 1, solve.parse()?))
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

    const SCRAMBLES: [&str; 10] = [
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

    #[test]
    fn roux_solve_round_trip() -> anyhow::Result<()> {
        let solves = (0..10)
            .map(|i| -> anyhow::Result<TimedSolve> {
                let scramble = Algorithm::from_static(SCRAMBLES[i]);
                let mut recon = Reconstruction::new(scramble.clone(), ValidMethods::Roux);
                recon.extend(
                    Roux::new()
                        .solve(
                            &mut Cube3x3::from_moves(&scramble.to_string())
                                .with_context(|| format!("{scramble:?}"))?,
                        )
                        .with_context(|| format!("{scramble:?}"))?
                        .iter()
                        .flat_map(|s| s.iter().copied()),
                );
                Ok(TimedSolve::new(
                    Reconstruction::new(scramble, ValidMethods::Roux),
                    Duration::from_millis(fastrand::u64(0..2 * 3_600_000)),
                ))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;
        let list = SolveList::from_solves(solves);

        assert_eq!(
            list,
            SolveList::from_str(&list.to_string())
                .with_context(|| format!("{:?}", list.to_string()))?
        );
        Ok(())
    }

    #[test]
    fn solves_path_pin() {
        assert_eq!(
            SolveList::solves_path().unwrap(),
            config_dir().unwrap().join("virtual_cube_timer/solves.txt")
        );
    }

    #[test]
    fn from_solves_and_from_solves_path_agree() -> anyhow::Result<()> {
        let path = PathBuf::from_str(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/solves.txt",
        ))?;

        let mut solves = SolveList::new();
        assert!(solves.is_empty());
        solves.extend(
            std::fs::read_to_string(&path)?
                .split_terminator("\n\n")
                .map(|solve| {
                    let (_number, no_number) =
                        solve.split_once('.').unwrap_or_else(|| panic!("{solve}"));
                    str::parse::<TimedSolve>(no_number)
                })
                .collect::<Result<Vec<_>, _>>()?,
        );
        assert!(!solves.is_empty());
        let indirect = SolveList::from_solves(solves.0.iter().map(|v| v.1.clone()));
        let direct = SolveList::from_solves_file(&path)?;
        assert_eq!(
            direct, indirect,
            "direct = \n{direct}\nindirect = \n{indirect}"
        );
        Ok(())
    }

    #[test]
    fn from_solves_file_errors_correctly() -> anyhow::Result<()> {
        let bad_path = PathBuf::from_str(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/error.txt"
        ))?;
        assert!(
            !bad_path.exists(),
            "this test is designed to start from a state where {} does not exist.\nDelete it to run the test, or ignore the test if it cannot be deleted.",
            bad_path.to_str().unwrap()
        );

        let error = SolveList::from_solves_file(&bad_path);
        assert_eq!(error?, SolveList::new());
        assert!(bad_path.exists());

        #[cfg(unix)]
        {
            use std::assert_matches;
            use std::fs::File;
            use std::os::unix::fs::PermissionsExt;

            let file = File::open(&bad_path)?;
            let mut mode = file.metadata()?.permissions();
            mode.set_mode(0o200);
            file.set_permissions(mode)?;
            drop(file);

            let error = SolveList::from_solves_file(&bad_path);
            assert_matches!(error, Err(_));
        }

        fs::remove_file(bad_path)?;
        Ok(())
    }
}
