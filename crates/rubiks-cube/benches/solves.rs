//! Benchmark: two passes of 1000 seeded random solves with default Roux, timed and
//! memory-tracked per step.
//!
//! Run with `cargo bench --bench solves`; results go to standard error. No logger is installed, so
//! the solver's log calls print nothing and cost only a level check. The first solve of each pass
//! is reported apart for timing and memory, but included in move counts. `harness = false`
//! in `Cargo.toml` makes this file a plain program, so
//! `cargo bench` works on stable.
//!
//! Memory is heap bytes requested through the global allocator, not what the OS reports.
//! "Peak" for a step is the highest heap in use during the step, minus the heap in use when it
//! started: the search's working memory plus whatever the memo grew. "Live" at the end of a
//! pass is what stays allocated between solves, which is mostly the memos.
#![expect(
    clippy::print_stderr,
    reason = "the benchmark reports its results on stderr"
)]
use std::{
    alloc::{GlobalAlloc, Layout, System},
    collections::BTreeMap,
    error::Error,
    num::TryFromIntError,
    sync::atomic::{AtomicUsize, Ordering::Relaxed},
    time::{Duration, Instant},
};

use rubiks_cube::{Cube3x3, Method, Puzzle, Roux, Solution};

const SOLVES: u32 = 1000;

/// Counts the bytes the program has allocated and the most it has held at once.
struct Counting;

static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

fn grew(bytes: usize) {
    let now = LIVE.fetch_add(bytes, Relaxed) + bytes;
    PEAK.fetch_max(now, Relaxed);
}

// SAFETY: every call forwards to `System` with the same arguments; the counters only observe.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: the caller upholds `alloc`'s contract, which is `System.alloc`'s.
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            grew(layout.size());
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: `ptr` came from this allocator, which is `System`, with this `layout`.
        unsafe { System.dealloc(ptr, layout) };
        LIVE.fetch_sub(layout.size(), Relaxed);
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: the caller upholds `realloc`'s contract, which is `System.realloc`'s.
        let new = unsafe { System.realloc(ptr, layout, new_size) };
        if !new.is_null() {
            LIVE.fetch_sub(layout.size(), Relaxed);
            grew(new_size);
        }
        new
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

/// Moves, times, and peak memory for one step or for whole solves.
#[derive(Default)]
struct Stats {
    lengths: Vec<usize>,
    times: Vec<Duration>,
    peaks: Vec<usize>,
}

/// One step's time and peak extra heap.
struct Measured {
    time: Duration,
    peak: usize,
}

fn main() -> Result<(), Box<dyn Error>> {
    let roux = Roux::default();
    // Pass 2 gets new scrambles but keeps the memos pass 1 grew, so what pass 2 no longer
    // pays for is memo growth.
    for (pass, seed) in [(1, 2026), (2, 2027)] {
        eprintln!("pass {pass} (seed {seed}):");
        bench(roux, seed)?;
    }
    eprintln!("peak heap over the whole run: {}", mib(PEAK.load(Relaxed))?);
    Ok(())
}

fn bench(roux: Roux, seed: u64) -> Result<(), Box<dyn Error>> {
    let mut rng = fastrand::Rng::with_seed(seed);
    let mut whole = Stats::default();
    let mut per_step: BTreeMap<String, Stats> = BTreeMap::new();
    let start = Instant::now();

    for i in 0..SOLVES {
        let scrambled = Cube3x3::scrambled_with_seed(rng.u64(..));
        let mut cube = scrambled;
        let t = Instant::now();
        let (solution, measured) = solve_measured(roux, &mut cube)?;
        let elapsed = t.elapsed();
        if !scrambled.apply_moves(&solution.to_string())?.is_solved() {
            return Err(format!("solve {i} does not solve its scramble").into());
        }
        if i == 0 {
            eprintln!("  first solve: {elapsed:.2?}");
            for (segment, m) in solution.iter().zip(&measured) {
                eprintln!(
                    "    {}: {:.2?}, peak {}",
                    segment.name(),
                    m.time,
                    mib(m.peak)?
                );
            }
        }
        let mut total = 0;
        for (segment, m) in solution.iter().zip(&measured) {
            let stats = per_step.entry(segment.name().to_string()).or_default();
            stats.lengths.push(segment.moves().len());
            if i != 0 {
                stats.times.push(m.time);
                stats.peaks.push(m.peak);
            }
            total += segment.moves().len();
        }
        whole.lengths.push(total);
        if i != 0 {
            whole.times.push(elapsed);
            whole
                .peaks
                .push(measured.iter().map(|m| m.peak).max().unwrap_or_default());
        }
    }

    eprintln!("  wall time: {:.2?}", start.elapsed());
    eprintln!("  live heap at end of pass: {}", mib(LIVE.load(Relaxed))?);
    eprintln!("  timing and memory: solves 2-{SOLVES}; move counts: all {SOLVES} solves:");
    eprintln!("    whole solve: {}", summary(&mut whole)?);
    for (name, stats) in &mut per_step {
        eprintln!("    {name}: {}", summary(stats)?);
    }
    Ok(())
}

/// Drives `Method::solve_steps`, timing each `next()` and recording its peak extra heap.
/// Each `next()` runs exactly one step.
fn solve_measured(
    roux: Roux,
    cube: &mut Cube3x3,
) -> Result<(Solution<Cube3x3>, Vec<Measured>), Box<dyn Error>> {
    let mut steps = roux.solve_steps(cube);
    let mut segments = Vec::new();
    let mut measured = Vec::new();

    loop {
        let before = LIVE.load(Relaxed);
        PEAK.fetch_max(before, Relaxed);
        let peak_before = PEAK.swap(before, Relaxed);
        let t = Instant::now();
        let next = steps.next();
        let time = t.elapsed();
        let peak = PEAK.fetch_max(peak_before, Relaxed).saturating_sub(before);
        let Some(segment) = next else { break };
        measured.push(Measured { time, peak });
        segments.push(segment?);
    }
    Ok((segments.into_iter().collect(), measured))
}

/// Bytes as MiB with one decimal.
fn mib(bytes: usize) -> Result<String, TryFromIntError> {
    let kib = u32::try_from(bytes / 1024)?;
    Ok(format!("{:.1} MiB", f64::from(kib) / 1024.0))
}

/// Time, moves, and peak extra heap, summarized.
fn summary(stats: &mut Stats) -> Result<String, TryFromIntError> {
    stats.times.sort_unstable();
    stats.peaks.sort_unstable();
    let count = u32::try_from(stats.times.len())?;
    let move_count = u32::try_from(stats.lengths.len())?;
    let at = |len: usize, p: usize| len.saturating_sub(1) * p / 100;
    let time_at = |p| {
        stats
            .times
            .get(at(stats.times.len(), p))
            .copied()
            .unwrap_or_default()
    };
    let peak_at = |p| {
        stats
            .peaks
            .get(at(stats.peaks.len(), p))
            .copied()
            .unwrap_or_default()
    };
    let moves = u32::try_from(stats.lengths.iter().sum::<usize>())?;
    Ok(format!(
        "time mean {:.2?}, median {:.2?}, p99 {:.2?}, max {:.2?} ({count} runs) | peak median {}, max {} | moves mean {:.3}, max {} ({move_count} runs)",
        stats.times.iter().sum::<Duration>() / count.max(1),
        time_at(50),
        time_at(99),
        time_at(100),
        mib(peak_at(50))?,
        mib(peak_at(100))?,
        f64::from(moves) / f64::from(move_count.max(1)),
        stats.lengths.iter().max().copied().unwrap_or_default(),
    ))
}
