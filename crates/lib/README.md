# rubiks-cube

![CI](https://github.com/Iurig/rubiks-cube/actions/workflows/lib.yml/badge.svg)
![License](https://img.shields.io/github/license/Iurig/rubiks-cube)

A Rust library that models the 3×3×3 Rubik's Cube and solves it the way a person would, one named
step at a time. Its solving methods are Roux, ZZ, and Kociemba. More methods and other cube types
are planned.

A cube state is a value of type `Cube3x3`. Moves are cube states too, and applying a move is just
group multiplication. Everything is `Copy`, and the move table is built on first use.

> **Status: Initial Release** The core group structure and every move in standard notation
> (faces, slices, rotations, wide moves) work and are tested against real reconstructions. The Roux,
> ZZ, and Kociemba solvers work, and their public APIs are still changing. See
> [Current state](#current-state).

## Quick example

```rust
use rubiks_cube::{Cube3x3, Inv, Pow, Puzzle};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Apply a sequence to the solved cube, example is Sebastiano Tronto's 16 move FMC WR
    let scramble = Cube3x3::from_moves("R' U' F D2 L2 F R2 U2 R2 B D2 L B2 D' B2 L' R' B D2 B U2 L U2 R' U' F")?;
    let solution = Cube3x3::from_moves("D2 F' D2 U2 F' L2 D R2 D B2 F L2 R' F' D U'")?;

    // Composition is multiplication: left operand first, then right
    assert!((scramble * solution).is_solved());

    // Inverses and powers
    assert_eq!(Cube3x3::from_moves("R")?.inv(), Cube3x3::from_moves("R'")?);
    assert!(Cube3x3::from_moves("R U R' U'")?.pow(6).is_solved());

    // Chain moves onto an existing state
    let cube = Cube3x3::IDENTITY.apply_moves("R U")?.apply_moves("R' U'")?;
    assert_eq!(cube, Cube3x3::from_moves("R U R' U'")?);

    // Unknown tokens are errors, not panics
    assert!(Cube3x3::from_moves("R Q U").is_err());
    Ok(())
}
```

## Notation

Move strings use standard cube notation. Whitespace and newlines separate moves, and anything after
`//` on a line is a comment, so you can paste annotated reconstructions directly:

```rust
let solved = rubiks_cube::Cube3x3::from_moves("
    U' L2 D' B2 D R2 F2 D' B2 R2 D B' R F2 R D' B' F U2 R' U D   // scramble
    y2 F' M F' R U' R U' Fw z'                                  // FB
    U R U r M' U' R U2' R'                                      // SS
    ...
");
```

| Kind        | Tokens                    | Implemented |
| ----------- | ------------------------- | ----------- |
| Face turns  | `R` `L` `U` `D` `F` `B`   | Yes         |
| Slice moves | `M` `E` `S`               | Yes         |
| Rotations   | `x` `y` `z`               | Yes         |
| Wide moves  | `Rw` … or lowercase `r` … | Yes         |

All these work with the following 4 modifiers: ` `(none), `'`, `2`, `2'`, and `'2`.

A lowercase face letter (`r`, `l`, `u`, `d`, `f`, `b`) is the wide move of that face, so `r` and
`Rw` are the same move. Every token goes through the same path, so a lone `r` or a line that is only
a comment behaves the same as it would inside a longer sequence. A string with no moves in it, such
as an empty string or a comment on its own, leaves the cube unchanged. A token that is not a move
makes the whole call return an `Err` naming that token.

## Solving

`Roux`, `ZZ`, and `Kociemba` are the crate's methods, and all implement the `Method` trait. `solve`
runs a method's steps in order and returns a `Solution`, with one segment per step:

```rust
use rubiks_cube::{Cube3x3, Method, Puzzle, Roux};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let scramble = "D2 F2 R2 U L2 D R2 U' B2 L2 B L2 F' L D2 U R' B D2";
    let roux = Roux::default();

    let mut cube = Cube3x3::from_moves(scramble)?;
    let solution = roux.solve(&mut cube)?;
    assert!(cube.is_solved());

    // A printed solution is notation, so the scramble followed by it replays to solved.
    let replayed = Cube3x3::from_moves(scramble)?.apply_moves(&solution.to_string())?;
    assert!(replayed.is_solved());

    // The same solve, one step per `next()`.
    let mut cube = Cube3x3::from_moves(scramble)?;
    for segment in roux.solve_steps(&mut cube) {
        print!("{}", segment?);
    }
    Ok(())
}
```

A printed solution has one line per step: the moves, then `//` and the step's name. One run of
[examples/main.rs](crates/rubiks-cube/examples/main.rs) printed this for the scramble above:

```text
U' F' B2 D R2 B D2    // FB
U2 R2    // DR
R2 U' R U R' U R'    // SB Square
U M' U2 Rw2 R U' Rw    // SB Pair
U2 R U R' U' R' F R F' R U R' U R U2 R' U'    // CMLL
U2 M' U M' U2 M U2 M' U' M' U2 M'    // LSE
```

`Roux`'s builder methods pick how each stage is split. `first_block` and `second_block` choose
between one search, a square plus a pair, or an edge, then a square, then a pair, `cmll` chooses
between one algorithm and two, and lse only has eolr for now. With a split block the solver
tries both the front and the back square and keeps the shorter:

```rust
use rubiks_cube::{Roux, roux};

let roux = Roux::default()
    .first_block(roux::FirstBlock::SquarePair)
    .cmll(roux::Cmll::TwoLook);
```

A `Method` is a builder for a `Technique`, which is a list of steps. The crate has two step types:
`SearchStep`, which searches for a goal within a fixed set of moves or algorithms, and `Shortest`,
which keeps the shortest of several steps. Any type that implements the `Step` trait can join them.
The [API documentation](#api-documentation) describes each type.

`cargo bench --bench solves` times the solving time for a solve that fully builds all the heavy tables for each step (dominated by the builds, proxy for building tables time), as well as the time for a solve with a warm table.

## Features

- `anstyle` for printing the 3x3 cube with color.

```rust
Cube3x3::default().
```

## How it works

The cube is the direct product of three independent piece groups:

- **Centers**: 6 pieces, no orientation. Only slice moves and rotations move them. The other pieces
  are expressed as if these were not moved (i.e. moves that move them will be translated to face
  turns).
- **Corners**: 8 pieces, orientation in ℤ/3ℤ.
- **Edges**: 12 pieces, orientation in ℤ/2ℤ.

Each is a `PieceConfiguration<P, N, O>`: a permutation array of `N` pieces of type `P` plus an
orientation array of `N` values in `Zn<O>`. Composition (`then`) permutes and adds orientations;
inversion negates them. `Cube3x3` just composes its three configurations component-wise.

Piece types are fieldless `#[repr(u8)]` enums implementing the `Piece` trait, which lets the library
index arrays by piece at zero cost (a compile-time assertion checks that each enum's discriminants
match its `ALL` order). Ordering conventions follow blindfolded-solving standard memorization order,
except for edges which are clockwise by layer, starting on the last and going to the first:

- Centers: `U F R B L D`
- Corners: `UBL UBR UFR UFL DFL DFR DBR DBL`
- Edges: `UB UR UF UL FL FR BR BL DF DR DB DL`

Corner orientation counts clockwise twists relative to the U/D sticker being in the U or D layer;
edge orientation is 0 for oriented, 1 for flipped.

`Cube3x3` implements `Display` as the unfolded sticker net, each facelet lettered by the face its
sticker belongs to when solved (`println!("{cube}")`); `cube.facelets()` gives the same data as a
`Facelets` value with a `face(Center)` accessor. This is the debugging view: it shows what a
physical cube would look like, so a wrong twist or flip convention is visible at a glance.

Only the nine base moves (six faces, three slices) are written by hand, as explicit cycles in
[moves/table.rs](crates/rubiks-cube/src/puzzles/cube3x3/moves/table.rs). Rotations (`x = R M' L'`,
`y = U E' D'`, `z = F S B'`), wide moves (a face turn followed by its parallel slice), and every
inverse and double are derived from those nine, on first use, into a single `ALL_MOVES` table that
string parsing looks up. A derived move therefore cannot drift from its base moves.

## API documentation

Every public item has a doc comment. Build and open the documentation with:

```sh
cargo doc --open
```

The main entry points:

- **`Cube3x3`** is a cube state. `from_moves` and `apply_moves` apply notation, `*` composes states,
  and `corners()`, `edges()`, and `centers()` give the pieces for queries such as `piece_at`.
- **`Puzzle`** is the trait the solver works through. Import it to call `scramble` or
  `scramble_with_seed`.
- **`Method`** is the trait of a solving method, such as **`Roux`** or **`Kociemba`**. It turns the
  method's options into a list of steps, and `solve` or `solve_steps` runs them.
- **`Step`**, **`SearchStep`**, and **`Shortest`** are the steps a method is built from, and
  **`Mask`** is a partial representation of a puzzle - it's what most steps operate on and try to solve. **`PieceSet`** is the immovable "label like" counterpart used to define goals.
- **`Solution`**, and **`SolveError`** are what a solve returns.

## Project layout

The repository is a Cargo workspace. Commands run from the root cover every crate.

```text
Cargo.toml                    the workspace: its members and the shared dev profile
crates/
  rubiks-cube/                the library
    src/
      lib.rs                  public exports
      error.rs                all error types
      ops.rs                  Inv and Pow traits
      zn.rs                   Zn<N>
      piece.rs                sealed Piece trait, PieceConfiguration with the piece_at /
                              orientation_at queries and the permutation parity
      fast_hash.rs            FxHasher, the fast hasher behind the search memo
      indexed.rs              the Indexed trait
      permutation.rs          the Permutation struct
      ordinal.rs              simple print helper for ordinal numbers
      puzzles/
        mod.rs                the Puzzle trait
        label.rs              Labeled, Mask, PieceSet: which pieces must be in place or oriented
        algorithm.rs          Algorithm: a sequence of moves
        algset.rs             AlgSet: move sequences a search applies as single units
        cube3x3/
          mod.rs              Cube3x3, Mul/Inv/Pow impls, accessors, rotation-aware
                              is_solved, is_reachable
          pieces.rs           piece enums, counts, and type aliases for the 3×3
          facelets.rs         Facelets: the sticker net view, Display for Cube3x3, twist and
                              flip conventions as face tables; its tests pin the net against
                              face turns
          moves.rs            Move type, MovablePart/MoveModifier enums, move-sequence parsing
                              and printing
          widget.rs           ratatui optional widget
          moves/
            table.rs          ALL_MOVES table: 9 hand-written face and slice moves, rotations
                              and wide moves derived from them, then inverses and doubles of
                              everything; its tests pin the derivations only
      methods/
        mod.rs                the Method trait, Technique and its solve loop
        context.rs            the cache context definition and a global context to be shared
        step.rs               the Step trait
        solution.rs           Solution, Segment
        neutrality.rs         the Neutrality implementation, WIP.
        step/
          search_step/
            mod.rs            SearchStep: a forward search that meets the memo in the middle
            memorization.rs   BfsMemo: the backward search from a step's goal, kept across
                              solves
          choose/
            mod.rs            Shortest: runs each alternative, keeps the one with the fewest
                              moves
          combine_pruned/
            mod.rs            PrunedCombine: a search guided by prune tables
            prune_table.rs    PruneTable
        test_steps.rs         FixedStep, a hand-written step for tests (compiled only in tests)
        cube3x3.rs            the modules for 3x3 methods
        cube3x3/
          roux.rs             Roux and its options
          zz.rs               ZZ and its options
          kociemba.rs         Kociemba's two-phase method
          helpers.rs          helpers for defining methods and steps
          algsets/            algorithm lists shared between methods
            cmll/             CMLL
            oll/              OLL
            pll/              PLL
            zbll/             ZBLL
    tests/
      api/                    integration tests, one binary with a module per concern:
                              mask, algebra (group laws), moves (handedness pins for each
                              base move, move orders), notation (parsing and its errors),
                              reconstructions (real solves), solving (every Roux option
                              combination on seeded random scrambles), scramble, and stats
      tables/                 heavier tests: Kociemba and scrambling
    benches/
      solves.rs               warm and cold cache solves for every step
    examples/
      main.rs                 solves one scramble with two option sets and prints the solutions
      sketch.rs               a design sketch
      DESIGN.md               a document for writing things down while writing to sketch.rs
  virtual-cube-timer/         the command-line tool
  twisty-puzzles-py/          exported python library
    src/
      lib.rs
    python/
tools/
  cargo-bin/                  tool for automatically downloading and executing formatting bins
```

## Building and testing

Requires a Rust toolchain with the 2024 edition.

### Required Dependencies

The library depends on `fastrand` for random cube states.
It also depends on `log`, through which the solver reports each step at the `debug` level.
It never prints.
`rayon` runs the searches and the prune-table builds in parallel.
`thiserror` helps define errors.
`itertools` makes some iterator method chains less verbose.
`dyn-clone` allows for cloning dyn types internally.

None of their types are returned.

### Optional Dependencies

For feature `anstyle`:

- `anstyle` is used for simple colored output of the cubes for easier debugging.
- `termprofile` is turned on to automatically downgrade the colors as your terminal needs it.

For feature `ratatui`:

- `ratatui` and `ratatui-core` are turned on for the widget itself, the `anstyle` feature is also turned on.

Both features turn on public interface with dependency types.

```sh
cargo build
cargo test
cargo bench --bench solves
RUST_LOG=rubiks_cube=debug cargo run --example main
cargo run -p virtual-cube-timer -- --method roux --number 3
```

## Contributing

You can find more information about contributing in [CONTRIBUTING.md](CONTRIBUTING.md)

## Current state

All unit, integration, and doc tests pass; only one is `#[ignore]`d due to accessing an external file.

What works today:

- All six face turns, the three slice moves `M`, `E`, `S`, the three rotations `x`, `y`, `z`, and
  the six wide moves `Rw` ... `Bw` (also as lowercase `r` ... `b`), each with the `'`, `2`, and `2'`
  modifiers. Rotations and wide moves are derived on first use from the face and slice moves, so
  they cannot drift out of sync with them. The base moves are pinned by handedness tests, and the
  whole notation is exercised by CFOP and Roux reconstructions.
- Move strings return `Result`: an unknown token is an `Err` naming it, never a panic.
- `is_solved` is rotation-aware: it re-orients the cube by its centers before comparing with the
  identity, so `Cube3x3::from_moves("x y2 z'")` reports solved.
- A cube state can be queried: `cube.corners().piece_at(Corner::Ubr)` says which piece sits in the
  UBR slot, and `orientation_at` reads its twist or flip. The handedness tests are written against
  these queries, from outside the crate.
- `is_reachable` checks the twist, flip, and permutation-parity invariants, plus that the centers
  form a whole-cube rotation, so a state no move sequence can produce is rejected.
- The Roux solver works with every combination of its options. A test runs all 18 combinations of
  first block, second block, and CMLL on four seeded random scrambles and replays each solution, and
  the benchmark solves 200 more.
- ZZ solves with an EO line, F2L, then the last layer as OCLL and PLL, or in one look with ZBLL. A
  test runs every option combination on seeded random scrambles.
- Kociemba's two-phase method solves scrambles: phase 1 brings the cube into the domino subgroup
  with face turns, and phase 2 solves it with moves that stay there. Both phases are searches guided
  by prune tables, which the first solve builds in a few seconds.
- A solve fails with a `SolveError` naming the step it stopped at. Its kind says why: the step's
  requirements weren't met, the step reported its own `StepError`, or the step returned without
  meeting its goal.
- A method can mix step types, and `Shortest` picks between alternatives by move count.

## Roadmap

- Color neutrality. Necessary for recognition of rotation independent steps, so its implementation is necessary for CFOP solves.
- Fingertrick-aware output. A solution already prints as notation; this is why `2'` is kept distinct.
  from `2` in the `Move` label even though they share a cube state.
- More methods (CFOP, Petrus), and Kociemba in the command-line solver.
- Reuse `PieceConfiguration` / `Piece` for other puzzles.

## License

This project is licensed under the MIT License. See [LICENSE](./LICENSE) for the full text.
