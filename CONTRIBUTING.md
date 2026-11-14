# Contributing to rubiks-cube

First of all, welcome! This is my first personal programming project, which I use for studying Rust and programming in general. Contributions are very much welcome, and don't hesitate on making a PR or issue if in doubt of anything.

Issues of any kind are welcome: along with the usual fanfare, I'd gladly take any feedback you may have.
API design, code organization, behavior, internal details, are all areas I'm improving as I go, and, as such, shouldn't be treated as "set in stone".

### The Library

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

## API Documentation

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

## Project Layout

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

## Building and Testing

Requires a Rust toolchain with the 2024 edition.

To install the pre-commit hooks used, make sure you have [uv](https://docs.astral.sh/uv/#highlights) installed, then run

```bash
uv tool install pre-commit
pre-commit install
```

### Required Dependencies

The library depends on `fastrand` for random cube states.
It also depends on `log`, through which the solver reports each step at the `debug` level.
It never prints.
`rayon` runs the searches and the prune-table builds in parallel.
`thiserror` helps define errors.
`itertools` makes some iterator method chains less verbose.
`dyn-clone` allows for cloning `dyn` types internally.

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

## LLM Policy

Because one of the central purposes of the repo is my learning, I plan on reading and reviewing every contribution.
This means "vibe coding", in its purest form, is disallowed here.
This is not to mean you can't use LLMs code or help - but that you must understand every line you contribute yourself, even if you weren't the one that wrote it.
If there's too much code for you to review, there's certainly too much code for me to review.

This policy wasn't followed to the tee by me before it existed.
I tried to keep LLM help I was fine not reviewing scoped to certain parts of the codebase, but with time it became clear how sometimes, certain pieces of the codebase become more important than predicted, and how high friction going back to understand it is.
This is why this policy doesn't have the same exceptions I gave myself: I don't plan on having them going forward.
