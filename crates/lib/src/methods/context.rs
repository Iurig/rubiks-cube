use std::{
    any::{Any, TypeId},
    sync::{Arc, LazyLock, Mutex},
};

use crate::fast_hash::FxMap;

pub type UntypedCache = Arc<dyn Any + Send + Sync>;

/// Stores the caches for [`Step`](crate::Step)s that need one.
///
/// Can be used to provide your own caches for built in and created [`Step`](crate::Step)s.
///
/// # Examples
///
/// ```
/// use std::sync::{
///     Arc,
///     atomic::{AtomicUsize, Ordering},
/// };
///
/// use rubiks_cube::BuildContext;
///
/// #[derive(Default)]
/// struct NodeCounter(AtomicUsize);
///
/// let ctx = BuildContext::default();
///
/// // The first access creates the cache, later ones share it.
/// let a = ctx.get_or_create::<NodeCounter>();
/// let b = ctx.get_or_create::<NodeCounter>();
/// a.0.fetch_add(1, Ordering::Relaxed);
/// assert!(Arc::ptr_eq(&a, &b));
/// assert_eq!(b.0.load(Ordering::Relaxed), 1);
///
/// // A separate context holds its own caches.
/// let other = BuildContext::default();
/// assert_eq!(other.get_or_create::<NodeCounter>().0.load(Ordering::Relaxed), 0);
/// ```
#[derive(Default, Debug)]
pub struct BuildContext {
    caches: Mutex<FxMap<TypeId, UntypedCache>>,
}

impl BuildContext {
    /// Creates a new empty [`BuildContext`].
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Used to create a cache in a given [`BuildContext`].
    ///
    /// # Panics
    /// Panics if the [`Mutex`] is poisoned.
    pub fn get_or_create<C: Any + Default + Send + Sync>(&self) -> Arc<C> {
        let mut map = self
            .caches
            .lock()
            .expect("the mutex should not be poisoned");
        map.entry(TypeId::of::<C>())
            .or_insert_with(|| Arc::new(C::default()))
            .clone()
            .downcast::<C>()
            .expect("entry keyed by TypeId::of::<C>() always holds a C")
    }
}

/// The memos of every [`Method`](crate::Method) of every [`Puzzle`](crate::Puzzle), shared for the
/// whole program.
///
/// Two steps of the same type share a cache, even if used by different methods. Is used by default
/// for all methods that don't ask for a cache, but might need it.
pub static GLOBAL: LazyLock<BuildContext> = LazyLock::new(Default::default);
