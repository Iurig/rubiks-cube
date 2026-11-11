# Build context persistence

**Status:** tentative, not implemented. **Scope:** `BuildContext`, the caches it holds, and the
`serde` feature.

## 1. Overview

With the `serde` feature on, a `BuildContext` can be saved to bytes and loaded back. Every cache that
opts in is saved, whatever its type, including generic ones like `SearchCache<P>` and the cache for
`PrunedCombine`. Caches that don't opt in still work as they do now; saving skips them.

Each cache describes itself through serde. The context doesn't implement `Serialize` or
`Deserialize`. It has its own save and load methods in one binary format picked by the library
(section 5).

Type information only exists in `get_or_create::<C>()`, where `C` is known. So all type-specific
work happens there: decoding on the way in (section 3), and recording how to encode on the way out
(section 4). There is no registry of cache types.

## 2. Opting in

A cache that can be saved implements a trait, available only with the feature, that requires:

- `Serialize`, `DeserializeOwned`, `Default`, `Send`, `Sync`, `'static`;
- a stable name (section 6).

Persistable caches are fetched through a separate method, also feature-only, with those bounds.
Plain `get_or_create` keeps its current bounds and stores no encoder.

Library steps call the persistable method under `cfg(feature = "serde")` and the plain one
otherwise. That is the same condition the cache's own derive uses.

## 3. Loading: decode on first use

Loading decodes nothing. It reads the file into raw entries, *name → bytes*, kept in the context
next to the live map.

The persistable method, called with `C`:

1. Looks up `TypeId::of::<C>()` in the live map. If found, returns it, as now.
2. Otherwise, looks for raw bytes under `C`'s name. If found, decodes them as `C`, removes the raw
   entry, and inserts the result into the live map.
3. Otherwise, creates `C::default()`, as now.

If decoding fails, the cache starts empty and gets rebuilt, since every cache can be rebuilt.

## 4. Saving: an encoder stored with each entry

A live entry is the `Arc<dyn Any + Send + Sync>` plus an optional encoder. `UntypedCache` goes back
to being the bare `Arc` with that encoder next to it. The `Cacheable` trait isn't needed.

The encoder is a plain function pointer, `fn(&(dyn Any + Send + Sync)) -> Vec<u8>`. It comes from one
generic function that downcasts to `C` and serializes it. Naming that function with `C` gives a
pointer to that one compiled copy, which the persistable method stores when it creates the entry.

Saving writes:

- every live entry that has an encoder, under its name;
- every raw entry nobody claimed this run, unchanged. A run that loads a file and only uses
  Kociemba still keeps the ZZ tables when it saves.

Entries without an encoder are skipped.

## 5. Format

One compact binary serde format, used for both the outer file and each entry's bytes (postcard is
one candidate). It isn't the user's choice: the raw entries in section 3 must be bytes in a known
format so they can be decoded later as `C`.

`BuildContext` loses its `derive(Serialize, Deserialize)`.

## 6. Names

The name is the key on disk.

- Not `TypeId`: it has no serde impls and can change between builds.
- Not `std::any::type_name`: its output isn't guaranteed stable, and it includes the module path,
  so moving a file would orphan saved caches.
- It covers the generic parameters, so it is a function, not a `const`. `SearchCache<P>` combines
  its own name with the puzzle's.
- It carries a version, bumped whenever the cache's layout changes.

## 7. Cargo

In `crates/rubiks-cube/Cargo.toml`:

```toml
[features]
serde = ["dep:serde", "serde/rc", "papaya/serde"]
```

- `dep:serde` turns on the optional dependency. Once `[features]` names `serde`, cargo no longer
  creates that feature by itself.
- `serde/rc` gives `Arc` its serde impls.
- `papaya/serde` lets `PruneTable` derive, since it wraps a `papaya::HashMap`.

`FxMap` fields need nothing extra: a `HashMap` deserializes with any hasher that implements
`Default`.

## 8. Open questions

- Sharing isn't preserved: two `Arc`s pointing at one value come back as two copies. Check whether
  the `PrunedCombine` cache shares a table between keys.
- What happens when a plain `get_or_create::<C>()` created an entry that the persistable method
  later asks for: attach the encoder then, or not.
