# Step logging

**Status:** proposal, not implemented. **Scope:** `run_steps` in `methods/mod.rs`,
`examples/main.rs`, and two README lines.

## 1. Overview

Every solve writes one `debug` line when a step finishes: the step's name, how long it took, how
many moves it found, and the moves. A step that fails writes one `debug` line with the reason.
Nothing else changes: no new public item, no new dependency.

```text
[DEBUG rubiks_cube::methods] FB: 7 moves in 1.31ms: U' F' B2 D R2 B D2
[DEBUG rubiks_cube::methods] DR: 2 moves in 85µs: U2 R2
[DEBUG rubiks_cube::methods] CMLL couldn't start because its requirements weren't met
```

## 2. Where the line is written

In `run_steps`, around the call to `step.solve`. Reasons:

- Both `Method::solve_steps` and `Technique::solve_steps` go through it, so one place covers every
  solve.
- It sees each step as a `dyn Step`, so a step type written outside the crate is reported too.
  Logging inside `SearchStep::solve`, `Shortest::solve`, and `PrunedCombine::solve` would miss
  those.
- It reports a `Shortest` once. Logging inside the step types would also report every alternative
  a `Shortest` tries, and the reader could not tell which one was kept.

`run_steps` returns a lazy iterator, so the line for a step appears when the caller's `next()`
runs that step, between the caller's own output for the step before and the step after.

## 3. What a line says

On success, from what `run_steps` already holds:

- the name: `step.name()`;
- the elapsed time: a `std::time::Instant` taken before `step.solve` and read after it;
- the move count: `solution.move_count()`;
- the moves: each segment's `moves()`, joined by a space. `Algorithm<P>` has `Display` for every
  puzzle. `Segment`'s own `Display` exists only for `Cube3x3`, and `run_steps` is generic, so the
  line can't use it.

On failure: the step's name and the `SolveErrorKind`, which already has a one-line `Display`. Not
the whole `SolveError`: for `Requirements` its `Display` adds the puzzle net, nine more lines.

The failure line stays at `debug`, not `warn` or `error`. The caller gets the `SolveError` as a
value and decides what it means. A library that also logs it at `error` reports it twice.

## 4. Cost when logging is off

Two facts about the `log` macros make this cheap:

- **Arguments are evaluated only when the level is enabled.** `log::debug!("{}", x.expensive())`
  expands to a level check with the formatting inside it, so `expensive()` doesn't run when
  `debug` is off. Anything computed in a `let` before the macro runs always. So the joined moves
  must be built inside the macro's arguments, not in a variable above it.
- **`log::log_enabled!(log::Level::Debug)`** gives the same check as a `bool`, for work that
  doesn't fit inside the arguments.

The two `Instant` readings are the only cost that remains with logging off: tens of nanoseconds
per step, against a search. `log_enabled!` could guard them, at the price of an `Option<Instant>`.
Recommendation: don't guard.

## 5. Levels

| Level   | What it shows                                                        |
| ------- | -------------------------------------------------------------------- |
| `debug` | one line per step (this proposal); one line per prune-table depth    |
| `trace` | one line per search round, from `SearchStep::search`                 |

The prune-table line stays at `debug`. A table build is rare, long, and the cost worth seeing, and
with it at `trace` it would drown in search rounds.

## 6. Who sees the lines

A log line carries a *target*, which defaults to the path of the module that wrote it:
`rubiks_cube::methods` for `run_steps`. `env_logger` filters by target prefix, so
`RUST_LOG=rubiks_cube=debug` shows these lines and the prune-table ones, and
`RUST_LOG=rubiks_cube::methods::step::search_step=trace` shows only the search rounds.

The library only writes to `log`. Lines go nowhere until the program installs a logger.
`examples/main.rs` installs none, so it needs `env_logger::init();` as the first line of `main`.
`env_logger` is already a dev-dependency.

## 7. README

- `README.md:285` then matches: "the solver reports each step at the `debug` level". Add that
  prune-table builds report each depth there too.
- `README.md:294` then prints the lines.

## 8. Decisions to make

1. **One line per step, or one per segment.** Per step (recommended) fits the elapsed time, which
   belongs to the step. Its weakness: a `Shortest` built with `Shortest::new` is named
   `"a or b"`, so the line doesn't say which alternative ran. Per segment uses each segment's
   name, which is the alternative's, but splits one time across lines and writes nothing for a
   step that returns no segments. Every `Shortest` the crate builds uses `Shortest::named`, so
   the weakness only reaches hand-built ones.
2. **Elapsed time in or out.** In (recommended): it is what `solve_steps` is documented for, and
   it separates a step that deepened its memo from a warm one.
3. **Testing.** `log` has one logger per process, so a test would have to install a capturing
   logger for the whole test binary. Recommendation: no test; check by running the example.
