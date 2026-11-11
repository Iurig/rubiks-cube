# Step builder accessors and fluent setters

**Status:** tentative, not implemented. Written to sit with before deciding. **Scope:** the
`StepBuilder` trait, `SearchStepBuilder`, and their callers in `TechniqueBuilder::finish_in` and
`All`.

## 1. Background

`StepBuilder` exposes its parameters through one accessor per parameter:

```rust
fn goal(&mut self) -> &mut Option<PieceSet<P>>;
fn assumed_solved(&mut self) -> &mut Option<PieceSet<P>>;
```

`SearchStepBuilder` also has an inherent fluent setter with the same name:

```rust
pub fn goal(&mut self, goal: PieceSet<P>) -> &mut Self;
```

This causes four problems:

- **Reading needs `&mut self`.** `All::can_solve` clones itself just to read `assumed_solved`.
- **`goal` means two things.** It sets on a concrete builder and gives mutable access through the
  trait.
- **The doc can't be enforced.** Builders that can't represent a goal "should always return `None`",
  but a caller can write `Some` through the `&mut Option`, and `finish_in` does. Every implementor
  also has to store that `Option`.
- **Chains end in a `&mut`.** `TechniqueBuilder::then` takes a builder by value, so call sites
  finish with `.clone()` or start with `*`.

## 2. Priorities

- People using the crate's own builders come before external implementors.
- Method chaining is worth keeping.
- The trait exists partly to unify the builder API, so the same setter shouldn't be repeated on each
  builder.
- About four implementors are planned. They are being held back while the API can still move, so
  implementor cost is a real concern and not a hypothetical one.
- Composite builders such as `All` have non-trivial accessors, not just `self.goal`.

## 3. Proposal

### Sketch

```rust
pub trait StepBuilder<P: Puzzle>: DynClone + Debug + Send + Sync {
    // Required
    fn get_goal(&self) -> Option<PieceSet<P>>;
    fn set_goal(&mut self, goal: PieceSet<P>); // see open question 1
    fn get_assumed_solved(&self) -> Option<PieceSet<P>>;
    fn set_assumed_solved(&mut self, pieces: PieceSet<P>);
    fn name(&self) -> Cow<'_, str>;
    fn allowed_sequences(&self) -> Option<AlgSet<P>>; // now `&self`
    fn build_in(&self, context: &BuildContext) -> Result<Arc<dyn Step<P>>, BuildStepError>;

    // Provided
    fn goal(mut self, goal: PieceSet<P>) -> Self
    where
        Self: Sized;
    fn assumed_solved(mut self, pieces: PieceSet<P>) -> Self
    where
        Self: Sized;
    fn build(&self) -> Result<Arc<dyn Step<P>>, BuildStepError>;
}
```

`SearchStepBuilder` implements the required methods and keeps its own consuming `free_algs`.
`expect_solved` and the inherent `goal` go away.

### Receivers

| Kind          | Examples                                 | Receiver    | Why                                        |
| ------------- | ---------------------------------------- | ----------- | ------------------------------------------ |
| Fluent setter | `goal`, `assumed_solved`, `free_algs`    | `mut self`  | The chain has to end with an owned builder |
| Read          | `get_goal`, `get_assumed_solved`, `name` | `&self`     | Only looks at the builder                  |
| Direct write  | `set_goal`, `set_assumed_solved`         | `&mut self` | Has to work through `dyn`                  |
| Finish        | `build`, `build_in`                      | `&self`     | Works through `dyn`, can build repeatedly  |

All fluent setters consume. A single `&mut Self` setter in a chain would bring back `.clone()`.

### Why these choices

- **`get_` prefix.** Rust usually avoids it, but builders are the common exception because the bare
  name belongs to the setter. `std::process::Command` has `arg` and `get_args`.
- **`where Self: Sized` on fluent setters.** A method that returns `Self` would stop the trait being
  used as `dyn StepBuilder` (E0038). The bound excludes just that method from `dyn`. `Iterator` does
  the same with `map` and `filter`.
- **Consuming setters.** They remove the `.clone()` and `*` at the end of chains. `.build()` still
  works at the end of a chain because it borrows the temporary.
- **Getters return owned values.** A composite that computes its goal, for example as the
  intersection of its children's goals, has nothing to borrow from. The clones this adds for stored
  values are not on the hot path, which is the search: they happen at build time, or once per
  builder when `All` is reached, where `build_in` already clones more.

## 4. Costs

- **Implementors write four accessor methods.** For composite builders, the setter and getter will
  walk the same structure with input and output swapped. This is real code repetition, and no good
  way out was found (see section 5).
- **`finish_in` gets longer.** `.goal().clone_from(...)` becomes a read from the next builder and a
  `set_goal`. Assigning `assumed_solved` is affected the same way if setters take a plain
  `PieceSet`.
- **Users need the trait in scope** to call `.goal(...)`. This affects examples, tests and doctests.
- **Fluent setters need ownership.** Code holding a `&mut` builder or a `Box<dyn StepBuilder>` uses
  `set_*` instead.
- **Builders in variables.** `b.free_algs(x);` becomes `b = b.free_algs(x);`, since `free_algs` has
  no `set_` version.
- **Two chaining styles in the crate.** `TechniqueBuilder` keeps `&mut Self`, which is fine because
  its chains end in `finish`, which takes `&self`.

## 5. Alternatives considered

| Option                                                        | Result                                                                                                                                                                                      |
| ------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Derive the getter from the setter, or the reverse             | Not possible. `&self` can't produce a `&mut`, a `&mut self` method can't serve as a getter, and Rust can't be generic over mutability. This is why std has `first` / `first_mut`.           |
| One `&mut Option` accessor per parameter (today)              | Reading needs `&mut self`, and the accessor exposes the field.                                                                                                                              |
| Shared `StepParams` struct with `params()` and `params_mut()` | Rejected. Only two required methods, but awful to write at call sites and leaky: it exposes the storage, forces the struct on every implementor, and fits computed composite values poorly. |
| Default `get_*` to `None` and `set_*` to doing nothing        | Rejected. A builder that forgets an override compiles and silently ignores its goal.                                                                                                        |
| Two traits, with the accessors on an optional one             | Probably not worth it. `finish_in` and `All` need the accessors, so they'd need a way to reach the second trait from a `dyn` builder. It only helps builders with no goal to represent.     |
| `macro_rules!` that generates the accessors                   | Rejected. Writing `self.goal` isn't the burden. The repetition is in composite builders, which a field-based macro can't generate.                                                          |
| Inherent fluent setters on each builder                       | Rejected. No trait import needed, but it repeats the setters and works against unifying the API.                                                                                            |
| Drop fluent setters                                           | Rejected. About 15 call sites would need `let mut` and separate statements, to save a few lines.                                                                                            |

## 6. Open questions

1. **Should `set_*` take `PieceSet<P>` or `Option<PieceSet<P>>`?** A plain `PieceSet` reads better
   but needs an `if let` in `finish_in`, because the next builder's assumption can be `None`. An
   `Option` also allows clearing a value, which `All` may or may not need.
2. **What does a builder that can't represent a goal do on `set_goal`?** Ignoring the write is the
   current plan. The alternative is returning an error, which would burden every other builder.
3. **Does anything else need to change for `All`?** It clones builders and calls `set_goal` and
   `build_in` during a solve. Its `is_done` currently calls a method `StepBuilder` doesn't have.

`step-builders.md` also says builders "take `&mut self` and don't consume themselves, which keeps
this cheap". That applies to its `TechniqueBuilder` clones and would need updating if this proposal
is adopted, although `finish_in` is unaffected because it only uses `set_*` and `&self` methods.
