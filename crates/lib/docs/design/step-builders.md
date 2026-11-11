# Step builders and build context

**Status:** rough design. Decisions are agreed; nothing is implemented yet. **Scope:**
`TechniqueBuilder`, `StepBuilder`, `StepCache`, and the search-memo cache in
`methods/step/search_step/memorization.rs`.

## 1. Background

A `Technique` is an ordered list of `Arc<dyn Step<P>>`. `Step` is an open trait: the crate has
`SearchStep` and `Shortest`, and users are invited to write their own kinds.

`TechniqueBuilder` is meant to assemble a technique from step _builders_ instead of from
already-built steps. That lets it wire each step to its neighbours. For example, a step's
prerequisite can be set from the previous step's goal, and the first and last steps can get special
handling. `finish` is still `todo!()`.

Today the builders and their caches are closed enums:

```rust
pub enum StepBuilder<P: Puzzle> { Search(SearchStepBuilder<P>) }
pub enum StepCache<P: Puzzle>   { Search(SearchCache<P>), None }
```

## 2. Goals

- Any `Step` kind, including one a user writes, can go through `TechniqueBuilder`.
- `finish` handles every step through one common API, based only on the step's position and its
  neighbours.
- Steps that search for the same thing still share a memo across techniques and methods, as they do
  now.
- A caller can build into a cache that is separate from the global one, e.g. for tests or to limit
  memory.

Not a goal: letting `finish` treat each kind of step differently.

## 3. Decision: `StepBuilder` is a trait

### Rationale

An enum is the better fit when the code using it needs different logic for each variant. `finish`
doesn't: it asks every builder the same questions. A closed enum would also break the first goal,
since a user can implement `Step` but couldn't add a variant.

### Sketch

```rust
pub trait StepBuilder<P: Puzzle>: Debug + Send + Sync {
    /// What this step guarantees once done, if it can say so as pieces.
    /// `None` for steps whose goal is only a predicate (`Step::is_done`).
    fn goal(&self) -> Option<PieceSet<P>>;

    /// Called by `TechniqueBuilder` with the previous step's goal.
    fn assume_solved(&mut self, pieces: PieceSet<P>);

    fn build_step(&self, ctx: &BuildContext) -> Result<Arc<dyn Step<P>>, StepError>;
}
```

The method list is provisional. It should be whatever `finish` needs from a step's neighbours, and
no more.

### Consequences

- `TechniqueBuilder` stores `Arc<dyn StepBuilder<P>>`. It keeps `Clone`, either by cloning a builder
  before calling `assume_solved` or by using `dyn-clone`. Builders already take `&mut self` and
  don't consume themselves, which keeps this cheap.
- `then` / `start_with` take `impl StepBuilder<P> + 'static` instead of `impl Into<StepBuilder<P>>`.
  The `From<SearchStepBuilder<P>>` impl is removed.
- The trait has no associated `Cache` type, because that would stop it being used as `dyn
  StepBuilder`. Builders get caches from `BuildContext`.
- The `StepCache` enum is removed.

### Alternative considered: keep the enum

This would allow exhaustive matching in `finish`. It was rejected because `finish` doesn't need to
treat kinds differently, and the enum shuts out user-written step kinds.

## 4. Decision: `BuildContext` is a struct holding a type map

### Rationale

Builders need somewhere to find shared caches. If `BuildContext` were a trait, every cache type
would need its own method on it (`fn search_cache(&self)`, …). A user-written step kind still
couldn't add one without editing the trait, so that would only move the closed-set problem from the
builder to the context. A map from type to cache is open without needing a trait.

### Sketch

```rust
#[derive(Default)]
pub struct BuildContext {
    caches: Mutex<FxMap<TypeId, Arc<dyn Any + Send + Sync>>>,
}

impl BuildContext {
    /// The context's `C`, created with `Default` on first use.
    pub fn get_or_default<C: Any + Default + Send + Sync>(&self) -> Arc<C>;
}
```

The key is the **cache type** (`TypeId::of::<C>()`). `SearchCache<Cube3x3>` and
`SearchCache<Cube2x2>` have different `TypeId`s, so the puzzle is already part of the key. Because
of this, `BuildContext` is not generic over `P`: one context serves every puzzle and every step
kind. Typed accessors such as `search_cache::<P>()` can be added on top as conveniences.

### Global context

This follows the existing `build` / `build_in` split:

```rust
static GLOBAL: LazyLock<BuildContext> = LazyLock::new(Default::default);

impl<P: Puzzle> TechniqueBuilder<P> {
    pub fn finish_in(&self, ctx: &BuildContext) -> Result<Technique<P>, TechniqueError>;
    pub fn finish(&self) -> Result<Technique<P>, TechniqueError> { self.finish_in(&GLOBAL) }
}
```

### Alternatives considered

| Option                                                             | Result                                                                                                     |
| ------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------- |
| No context, global caches only                                     | Simplest, but breaks the goal of building into a separate cache.                                           |
| `BuildContext` struct with fixed fields (`search: SearchCache<P>`) | Works for the crate's own steps, but user-written step kinds can't add caches, and it is generic over `P`. |
| `BuildContext` trait                                               | Has the same closed-set problem as the enum (see rationale).                                               |
| **Struct holding a type map**                                      | **Chosen.** Open to any cache type and any puzzle.                                                         |

## 5. Migration from the current code

`GLOBAL_MEMO` in `memorization.rs` already uses the type-map pattern (`FxMap<TypeId, Arc<dyn Any +
Send + Sync>>` plus a `downcast`), but it is keyed by puzzle type and always holds a
`SearchCache<P>`.

1. Change the key from `TypeId::of::<P>()` to `TypeId::of::<C>()`.
2. Move the map from the static into `BuildContext`. The static becomes `GLOBAL: BuildContext`.
3. Replace `global_cache::<P>()` with `GLOBAL.get_or_default::<SearchCache<P>>()`, and change
   `SearchStepBuilder::build_in` to take `&BuildContext`.
4. Turn `StepBuilder` into a trait, implement it for `SearchStepBuilder`, then delete the
   `StepBuilder` and `StepCache` enums.
5. Implement `finish_in` / `finish`.

Clean-up to do at the same time:

- The `SearchCache` doc comment ("Statics cannot be generic, so each puzzle type … keeps its own
  cache in a static") will be out of date.
- The `BuildContext` name is still used in `global_cache`'s `expect` message and in the
  `SearchCache::new` doc comment.
- Two bugs in the WIP `finish`: `last if last == self.step_builders.len()` never matches, and the
  `step` argument is never pushed.

## 6. Open questions

- What exactly does `finish` need from neighbours? Is `goal` + `assume_solved` enough? Does the last
  step need to be told that it is last?
- Error type: should `build_step` return `StepError`, or a new builder error that wraps
  `SearchStepError` and the errors of other step kinds?
- Does `Shortest` get a builder? If so, what is its `goal`: the intersection of its children's
  goals?
- Should `BuildContext` be cheap to clone (an `Arc` inside) so that callers can share it easily?
