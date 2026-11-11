# Step builder wiring

**Status:** tentative, not implemented. **Scope:** the `StepBuilder` trait,
`TechniqueBuilder::finish_in`, and `All`.

## 1. Overview

`StepBuilder` has one job: describe a step and build it. A builder reports what it knows through
getters and never gets changed by the code that wires it.

`finish_in` does the wiring. It reads every builder, works out each step's final goal and
prerequisite, and passes those values to the build call.

Hand-made values go in through setters on the concrete builder, before the builder reaches
`TechniqueBuilder`. Steps that need several builds, like `All`, do them at build time.

## 2. The trait

`build_wired` is a placeholder name.

Required:

- `name`: the name of the step being built.
- `get_goal`: the builder's goal, as an `Option<PuzzleCondition>` (section 3).
- `get_assumed_solved`: the builder's prerequisite, as an `Option<PuzzleCondition>`.
- `allowed_sequences`: the sequences a built step might use, for deriving a prerequisite.
- `build_wired`: builds from a goal and a prerequisite, each an `Option<PuzzleCondition>`, and a
  context.

Provided:

- `build_in`: calls `build_wired` with the builder's own values.
- `build`: calls `build_in` with the global context.

Every method takes `&self`. Getters return owned values, because a composite builder may compute its
answer and have nothing to borrow from. Unlike the `&mut Option` accessors, a getter can't be used to
write, and reading doesn't need a mutable builder.

There is no `set_goal` or `set_assumed_solved`. Since nothing mutates a builder through `dyn`,
`finish_in` no longer needs `arc_make_mut` or `split_at_mut`, and it looks like `DynClone` can go too.

## 3. `PuzzleCondition`

A `PuzzleCondition` is a goal or a prerequisite that a builder has. It is a struct with one private
field, which holds a private enum.

- `PuzzleCondition::pieces(set)` creates a condition expressed as pieces.
- `PuzzleCondition::opaque()` creates a condition that exists but can't be inspected. A future OCLL
  builder would use it for its goal, which holds up to an AUF. A `PrunedCombine` builder would use it
  for its prerequisite, which is a closure.
- `as_pieces()` returns the pieces, or `None` for anything else.

The `Option` around it says whether the builder has a value at all:

| Value                              | Meaning                             | `finish_in`              |
| ---------------------------------- | ----------------------------------- | ------------------------ |
| `None`                             | Unset                               | Fills it in              |
| `Some`, `as_pieces()` gives pieces | An explicit condition, as pieces    | Keeps it and checks it   |
| `Some`, `as_pieces()` gives `None` | A condition that can't be inspected | Keeps it, doesn't check  |

A step that really has no prerequisite reports empty `pieces`, not `None`. Otherwise it would get a
prerequisite derived from its moves.

### Future formats

The struct leaves room for conditions that aren't piece sets, such as a later `UpToAuf`. Adding one
takes a private variant, a constructor, and an accessor.

- Other crates can't match on the private enum, so moving a step from opaque to a new format can't
  change which arm of their code runs. A public `Opaque` variant, or a public enum with
  `#[non_exhaustive]`, would allow that. Rust has no private variants, and `#[doc(hidden)]` only
  hides one from the docs.
- Code that doesn't know a format sees `as_pieces()` return `None` and treats it as opaque. Every
  future format must therefore be safe to treat that way: it describes a condition that exists and
  must not be replaced. `None` around the struct stays the only way to ask for a value to be filled
  in.
- `finish_in` is inside the crate, so it matches the private enum exhaustively. Adding a format is a
  compile error there, which forces a decision about how to wire it.
- `build_wired` takes the same type, so wiring a new format doesn't change its signature.

The struct should exist before the first release, since moving to it later would be a breaking
change.

## 4. Setting values by hand

Each builder that really has a piece goal defines its own fluent `goal` and `assumed_solved` setters,
the same way `SearchStepBuilder` defines `free_algs`.

```text
.then(SearchStep::builder(...).goal(x))   get_goal reports Some(PuzzleCondition::pieces(x))
.then(SearchStep::builder(...))           get_goal reports None
```

- `.goal(x)` on a builder with no piece goal is a compile error, not a silent no-op.
- Using a builder's setters needs no trait import, unlike provided methods on the trait.
- The setters are repeated on each builder that has them, about 8 lines per builder and likely two or
  three builders. This gives up keeping one copy on the trait, which is acceptable because the trait
  serves generic callers such as `finish_in`. Fluent setters require `Self: Sized`, so those callers
  can't use them anyway.

## 5. How `finish_in` resolves a technique

`finish_in` walks backward, from the last step to the first, and does this for each builder:

1. **Resolve the prerequisite.**
   - First step: a prerequisite other than empty `pieces` is `InvalidFirstStep`, including an opaque
     one. An unset one stays `None`.
   - Any other step: keep a set one. If it is unset, derive `pieces` from `allowed_sequences`, or stay
     `None` if the builder has none.
2. **Resolve the goal.**
   - Last step: a set goal is `InvalidLastStep`, and the goal stays `None`.
   - Other steps:
     - **Unset:** takes the next step's resolved prerequisite if it is `pieces`. If it is opaque or
       `None`, that is an error, where today a `SearchStep` would silently solve the whole cube. The
       goal has to be set by hand.
     - **Pieces:** kept. If the next step's resolved prerequisite is `pieces`, the goal must cover it,
       checked with `is_subset`, or it is an error.
     - **Opaque:** kept unchecked, since there are no pieces to compare.
3. **Build** with `build_wired`, passing both resolved values and the context.
4. **Carry** this step's resolved prerequisite in a local variable for the next iteration, which
   handles the step before it. Today the value travels by being written into the next builder.

An explicit value wins over a filled-in one, as it does today.

The `is_subset` check in step 2 applies to derived prerequisites too, because `SearchStep` enforces
them the same way. A goal that doesn't cover the next step's prerequisite can't set the puzzle up for
that step, so it fails at build time instead of as a `Requirements` error during some solves. It is
the same rule `SearchStepBuilder::build_in` applies within one step, between `before` and `after`. A
hand-built technique that fails it has a wrong goal. If a derived prerequisite is stricter than
intended, the next step can set an explicit `assumed_solved`.

When either side is opaque there is nothing to check, so a wrong pairing still fails during a solve.

### What `build_wired` receives

- **Goal:** `None` only when no goal was given, which in a technique means the last step. The builder
  uses its default, and `SearchStep`'s default is to solve everything.
- **Prerequisite:** `None` on a first step with no prerequisite, or when nothing could be derived.
- **Opaque values:** a builder with an opaque condition gets its own value back.
- **Conditions it can't use:** a builder that needs pieces and gets something else returns
  `BuildStepError`.

## 6. `All`

`All` is an `AllBuilder` plus a small runtime step.

`AllBuilder` builds one step for each child and each set of already-solved children, up front in
`build_wired`. The runtime `All` step holds only those built steps and picks among them during a
solve.

- `is_done` and `can_solve` come from `Step`, so `StepBuilder` needs no `is_done`.
- The context is only needed at build time, so the step doesn't store one or fall back to `GLOBAL`.
- A solve doesn't clone or rebuild anything, and memos are shared across solves as usual.
- The count is at most n·2ⁿ⁻¹ steps, or 32 for four children. Each build clones its sets and looks up
  a memo, and memos stay empty until used.

`finish_in` treats `AllBuilder` like any other builder. Its `build_wired` combines the resolved goal
and prerequisite with its children's. If a child's condition has no pieces, `AllBuilder` reports
`opaque` itself, or returns `BuildStepError` if it can't combine them.

## 7. Costs

- Builders with a piece goal repeat their fluent setters.
- `build_wired` receives values that may differ from the builder's own fields.
- Reading a condition goes through accessors, not a `match`.
- `AllBuilder` builds every step up front, including ones a given solve never uses.
- Allowing empty `pieces` on a first step changes `starting_step_with_prereq_returns_error`, which
  expects `InvalidFirstStep` for `expect_solved(PieceSet::new())`.
- Either/or prerequisites stay inexpressible (see open question 3).

## 8. Open questions

1. **Which builders are planned, and how many can state their goal as pieces?** If only `SearchStep`
   and `All` can, it is worth asking whether `TechniqueBuilder` should assemble everything, since
   `Roux` and `ZZ` already mix in steps it can't wire.
2. **What does `All` mean?** Section 6 assumes "solve the children in any order, keeping what's done",
   because `All::solve` is still empty. If the goals can't be listed in advance, building up front
   doesn't work.
3. **How do either/or steps go through `finish_in`?** A `Shortest` prerequisite, such as front square
   or back square, isn't one `PieceSet`, and an intersection is weaker than what the next step needs.
   A `Shortest` builder could report an opaque goal and prerequisite, but then its neighbours can't be
   wired from it and must be explicit. The alternative is to keep `chain_steps!` and its before ×
   after cross product outside `TechniqueBuilder`.

If this is adopted, `step-builders.md` also needs updating where it says builders "take `&mut self`
and don't consume themselves".
