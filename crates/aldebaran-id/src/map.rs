//! Maps of [`trait@Id`]s to values.

use core::hash::{BuildHasher, BuildHasherDefault, Hash};

pub mod reverse;

use crate::map::reverse::ReverseMap;
use hashbrown::HashMap;
use rustc_hash::FxHasher;

use crate::prelude::Id;

/// A map from [`trait@Id`]s to values.
///
/// # Time Complexity
///
/// Operations that use the hash table have the same expected time complexity as
/// the underlying [`HashMap`]. Their actual cost also depends on hashing and
/// comparing identifiers.
///
/// Although lookups are expected to take `O(1)` time with ordinary hash
/// distribution, collisions can make them linear in the number of entries.
///
/// New IDs append to the aligned key and value slices, replacement preserves
/// a pair's position, and removal may move the final pair into that position.
///
/// # Remarks
///
/// The [`PartialEq`] and [`Eq`] implementations compare identifier-value
/// associations, independent of their storage order.
#[derive(Debug, Clone)]
// NOTE(invariant): ForAll((Id, Index) In Map): Index < Storage.Length And Storage.Keys[Index] = Id
pub struct IdMap<I, V, S = BuildHasherDefault<FxHasher>>
where
    I: Id,
    S: BuildHasher,
{
    /// The hash map that maps the [`trait@Id`]s to the indices of the values in
    /// [`IdMap::storage`].
    map: HashMap<I, usize, S>,

    /// The value storage.
    storage: ReverseMap<I, V>,
}

impl<I, V, S> PartialEq for IdMap<I, V, S>
where
    I: Id,
    V: PartialEq,
    S: BuildHasher,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let &Self { ref storage, .. } = self;

        let &Self {
            map: ref other_map,
            storage: ref other_storage,
            ..
        } = other;

        storage.len() == other_storage.len()
            && storage
                .iter()
                .all(|(id, value)| other_map.get(id).and_then(|&other_index| other_storage.get(other_index)) == Some(value))
    }
}

impl<I, V, S> Eq for IdMap<I, V, S>
where
    I: Id,
    V: Eq,
    S: BuildHasher,
{
}

impl<I, V, S> IdMap<I, V, S>
where
    I: Id,
    S: BuildHasher,
{
    /// Creates a new empty [`IdMap`].
    ///
    /// # Time Complexity
    ///
    /// The time complexity of this operation is `O(1)`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use aldebaran_id::map::IdMap;
    /// use aldebaran_id::Id;
    /// use aldebaran_id::ident::Id as _;
    ///
    /// Id!(
    ///     #[derive(Debug,  Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    ///     become TestId
    /// );
    ///
    /// TestId!(8);
    ///
    /// let mut map: IdMap<TestId8, &str> = IdMap::new();
    ///
    /// assert_eq!(map.len(), 0);
    ///
    /// assert_eq!(map.capacity(), 0);
    ///
    /// assert!(map.is_empty());
    ///
    /// assert_eq!(map.keys().count(), 0);
    ///
    /// assert_eq!(map.values().count(), 0);
    ///
    /// eprintln!("but, is it truly brand-new?: {:?}", map);
    ///
    /// map.clear(); // meh, it's empty anyway
    /// ```
    #[inline]
    pub fn new() -> Self
    where
        S: Default,
    {
        let map = HashMap::with_hasher(S::default());
        let storage = ReverseMap::new();

        Self { map, storage }
    }

    /// Creates a new empty [`IdMap`] with the given hasher.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use aldebaran_id::map::IdMap;
    /// # use aldebaran_id::Id;
    /// # use aldebaran_id::ident::Id as _;
    /// #
    /// # use std::collections::hash_map::RandomState;
    /// #
    /// # Id!(
    /// #   #[derive(Debug,  Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    /// #   become TestId
    /// # );
    /// #
    /// TestId!(8);
    ///
    /// let map: IdMap<TestId8, &str, RandomState> = IdMap::with_hasher(RandomState::new());
    ///
    /// assert_eq!(map.len(), 0);
    ///
    /// assert_eq!(map.capacity(), 0);
    /// ```
    #[inline]
    pub fn with_hasher(hasher: S) -> Self {
        let map = HashMap::with_hasher(hasher);
        let storage = ReverseMap::new();

        Self { map, storage }
    }
}

impl<I, V, S> Default for IdMap<I, V, S>
where
    I: Id,
    S: BuildHasher + Default,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<I, V, S> IdMap<I, V, S>
where
    I: Id,
    S: BuildHasher,
{
    /// Query the capacity of this [`IdMap`], that is, the maximum number of
    /// elements it can hold without reallocating.
    ///
    /// # Time Complexity
    ///
    /// The time complexity of this operation is `O(1)`.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use aldebaran_id::map::IdMap;
    /// # use aldebaran_id::Id;
    /// # use aldebaran_id::ident::Id as _;
    /// #
    /// # Id!(
    /// #   #[derive(Debug,  Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    /// #   become TestId
    /// # );
    /// #
    /// TestId!(8);
    ///
    /// let mut map: IdMap<TestId8, &str> = IdMap::new();
    ///
    /// assert_eq!(map.capacity(), 0);
    ///
    /// map.insert(TestId8::MIN, "hello");
    ///
    /// assert!(map.capacity() >= 1);
    /// ```
    ///
    /// The reported capacity is limited by both the hash table and the value storage.
    #[inline]
    pub fn capacity(&self) -> usize {
        let &Self { ref storage, ref map, .. } = self;

        storage.capacity().min(map.capacity())
    }

    /// Query the number of elements in this [`IdMap`].
    ///
    /// # Time Complexity
    ///
    /// The time complexity of this operation is `O(1)`.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use aldebaran_id::map::IdMap;
    /// # use aldebaran_id::Id;
    /// # use aldebaran_id::ident::Id as _;
    /// #
    /// # Id!(
    /// #   #[derive(Debug,  Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    /// #   become TestId
    /// # );
    /// #
    /// TestId!(8);
    ///
    /// let mut map: IdMap<TestId8, &str> = IdMap::new();
    ///
    /// assert_eq!(map.len(), 0);
    ///
    /// map.insert(TestId8::MIN, "hello");
    ///
    /// assert_eq!(map.len(), 1);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        let &Self { ref storage, .. } = self;

        storage.len()
    }

    /// Determine if this [`IdMap`] is empty.
    ///
    /// # Time Complexity
    ///
    /// The time complexity of this operation is `O(1)`.
    #[inline]
    pub fn is_empty(&self) -> bool {
        let &Self { ref storage, .. } = self;

        storage.is_empty()
    }

    /// Clear this [`IdMap`], removing all elements.
    ///
    /// Its capacity is retained so it can be reused for later entries.
    #[inline]
    pub fn clear(&mut self) {
        let &mut Self {
            ref mut storage,
            ref mut map,
            ..
        } = self;

        map.clear();
        storage.clear();
    }

    /// Determine if the target [`trait@Id`] is contained in this [`IdMap`].
    ///
    /// # Time Complexity
    ///
    /// On average, this operation takes `O(1)` time.
    #[inline]
    pub fn contains_key<Q>(&self, id: Q) -> bool
    where
        Q: AsRef<I>,
        Q: Eq + Hash,
    {
        let &Self { ref map, .. } = self;

        map.contains_key(id.as_ref())
    }

    /// Retrieve an iterator over the keys of this [`IdMap`].
    ///
    /// # Time Complexity
    ///
    /// Creating the iterator takes `O(1)` time. Key order is unspecified because
    /// it follows hash-table order.
    #[inline]
    pub fn keys<'a>(&'a self) -> impl Iterator<Item = &'a I> + 'a {
        let &Self { ref map, .. } = self;

        map.keys()
    }

    /// Retrieve an iterator over the values of entries of this [`IdMap`].
    ///
    /// # Time Complexity
    ///
    /// Creating the iterator takes `O(1)` time. Values follow paired storage
    /// order and need not match [`Self::keys`].
    #[inline]
    pub fn values<'a>(&'a self) -> impl Iterator<Item = &'a V> + 'a {
        let &Self { ref storage, .. } = self;

        storage.values().iter()
    }

    /// Retrieve a mutable iterator over the values of the entries of this
    /// [`IdMap`].
    ///
    /// # Time Complexity
    ///
    /// Creating the iterator takes `O(1)` time. Values follow the order of
    /// [`Self::as_keys`].
    #[inline]
    pub fn values_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut V> + 'a {
        let &mut Self { ref mut storage, .. } = self;

        storage.iter_mut().map(|(_, v)| v)
    }

    /// Retrieve a reference to a slice of the values of the entries of this
    /// [`IdMap`].
    ///
    /// # Time Complexity
    ///
    /// This operation takes `O(1)` time. Each value is aligned by index with
    /// [`Self::as_keys`].
    #[inline]
    pub fn as_values(&self) -> &[V] {
        let &Self { ref storage, .. } = self;

        storage.values()
    }

    /// Retrieve a reference to a slice of the keys of the entries of this
    /// [`IdMap`].
    ///
    /// # Time Complexity
    ///
    /// This operation takes `O(1)` time. Each ID is aligned by index with
    /// [`Self::as_values`].
    #[inline]
    pub fn as_keys(&self) -> &[I] {
        let &Self { ref storage, .. } = self;

        storage.keys()
    }

    /// Retrieve a reference to the value corresponding to the given [`trait@Id`].
    ///
    /// # Time Complexity
    ///
    /// On average, this operation takes `O(1)` time.
    /// An ID that is not present yields [`None`].
    #[inline]
    pub fn get<Q>(&self, id: Q) -> Option<&V>
    where
        Q: AsRef<I>,
        Q: Eq + Hash,
    {
        let &Self { ref map, ref storage, .. } = self;

        map.get(id.as_ref()).and_then(|&index| storage.get(index))
    }

    /// Retrieve a mutable reference to the value corresponding to the given
    /// [`trait@Id`] in this [`IdMap`].
    ///
    /// # Time Complexity
    ///
    /// On average, this operation takes `O(1)` time.
    /// An ID that is not present yields [`None`].
    #[inline]
    pub fn get_mut<Q>(&mut self, id: &Q) -> Option<&mut V>
    where
        Q: AsRef<I>,
        Q: Eq + Hash + ?Sized,
    {
        let &mut Self {
            ref map, ref mut storage, ..
        } = self;

        map.get(id.as_ref()).and_then(move |&index| storage.get_mut(index))
    }

    /// Insert a new entry into this [`IdMap`], potentially replacing an
    /// existing one.
    ///
    /// # Time Complexity
    ///
    /// This operation takes expected `O(1)` time and is amortized over growth.
    /// New IDs append to the paired slices. Replacing an ID keeps its position
    /// and returns the previous value.
    #[inline]
    pub fn insert(&mut self, id: I, value: V) -> Option<V> {
        let &mut Self {
            ref mut map,
            ref mut storage,
            ..
        } = self;

        let target_index = map.get(&id).copied();

        if let Some(target_index) = target_index {
            let (_, target_value) = storage.swap_at(target_index, (id, value));

            Some(target_value)
        } else {
            let target_index = storage.len();

            let _: Option<usize> = map.insert(id, target_index);

            storage.push(id, value);

            None
        }
    }

    /// Remove the entry corresponding to the given [`trait@Id`] from this [`IdMap`].
    ///
    /// This returns the stored identifier and value when the ID is present, or
    /// [`None`] when it is absent.
    ///
    /// Removing an entry may move the final stored pair into its position.
    ///
    /// # Time Complexity
    ///
    /// This operation takes expected `O(1)` time.
    #[inline]
    pub fn remove<Q>(&mut self, id: Q) -> Option<(I, V)>
    where
        Q: AsRef<I>,
        Q: Eq + Hash,
    {
        let &mut Self {
            ref mut map,
            ref mut storage,
            ..
        } = self;

        let target_index = map.remove(id.as_ref());

        target_index.map(|target_index: usize| -> (I, V) {
            let removed = storage.swap_remove(target_index);

            if let Some(&moved_id) = storage.get_key(target_index) {
                let _: Option<usize> = map.insert(moved_id, target_index);
            }

            removed
        })
    }
}

impl<I, V, S> Extend<(I, V)> for IdMap<I, V, S>
where
    I: Id,
    S: BuildHasher,
{
    #[inline]
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = (I, V)>,
    {
        for (id, value) in iter {
            let _: Option<V> = self.insert(id, value);
        }
    }
}

impl<I, V, S> AsRef<[V]> for IdMap<I, V, S>
where
    I: Id,
    S: BuildHasher,
{
    #[inline]
    fn as_ref(&self) -> &[V] {
        self.as_values()
    }
}

#[cfg(test)]
mod tests {
    //! Tests module for the [`map`] module.
    //!
    //! [`map`]: super

    use super::*;

    use crate::Id;
    use crate::zeroable::NonZeroed;

    use core::{
        hash::Hasher,
        panic::AssertUnwindSafe,
        sync::atomic::{AtomicUsize, Ordering},
    };

    use std::panic::{catch_unwind, resume_unwind};

    Id!(
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        become TestId
    );

    TestId!(8 become);

    static HASH_CALLS: AtomicUsize = AtomicUsize::new(0);
    static PANIC_HASH_CALL: AtomicUsize = AtomicUsize::new(usize::MAX);

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct PanicHashId(TestId8);

    impl AsRef<Self> for PanicHashId {
        fn as_ref(&self) -> &Self {
            self
        }
    }

    impl Hash for PanicHashId {
        fn hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            let hash_call = HASH_CALLS.fetch_add(1, Ordering::SeqCst);

            if hash_call == PANIC_HASH_CALL.load(Ordering::SeqCst) {
                resume_unwind(Box::new(()));
            }

            self.0.hash(state);
        }
    }

    impl Id for PanicHashId {
        type BackingPrimitive = u8;

        const MAX: Self = Self(TestId::MAX);

        const MIN: Self = Self(TestId::MIN);

        fn raw(value: NonZeroed<Self::BackingPrimitive>) -> Self {
            Self(TestId8::raw(value))
        }

        fn as_nonzeroed(&self) -> NonZeroed<Self::BackingPrimitive> {
            self.0.as_nonzeroed()
        }
    }

    struct PanicOnDrop(bool);

    impl Drop for PanicOnDrop {
        fn drop(&mut self) {
            if self.0 {
                resume_unwind(Box::new(()));
            }
        }
    }

    #[test]
    fn insert_get_replace_and_clear() {
        let mut map: IdMap<TestId8, &str> = IdMap::new();
        let id = TestId::MIN;

        assert!(map.is_empty());
        assert_eq!(map.get(id), None);
        assert!(!map.contains_key(id));

        assert_eq!(map.insert(id, "first"), None);
        assert_eq!(map.get(id), Some(&"first"));
        assert_eq!(map.insert(id, "second"), Some("first"));
        assert_eq!(map.get(id), Some(&"second"));
        assert_eq!(map.len(), 1);

        let capacity = map.capacity();
        map.clear();

        assert!(map.is_empty());
        assert_eq!(map.capacity(), capacity);
        assert!(!map.contains_key(id));
    }

    #[test]
    fn remove_only_entry() {
        let mut map: IdMap<TestId8, &str> = IdMap::new();
        let id = TestId::MIN;

        assert_eq!(map.remove(id), None);
        assert_eq!(map.insert(id, "only"), None);
        assert_eq!(map.remove(id), Some((id, "only")));
        assert!(map.is_empty());
        assert!(!map.contains_key(id));
    }

    #[test]
    fn remove_final_entry_from_nonempty_map() {
        let mut map: IdMap<TestId8, &str> = IdMap::new();
        let first_id = TestId::MIN;
        let final_id = TestId::MAX;

        map.insert(first_id, "first");
        map.insert(final_id, "final");

        assert_eq!(map.remove(final_id), Some((final_id, "final")));
        assert_eq!(map.len(), 1);
        assert_eq!(map.get(first_id), Some(&"first"));
    }

    #[test]
    fn remove_displaced_entry_reindexes_survivor() {
        let mut map: IdMap<TestId8, &str> = IdMap::new();
        let removed_id = TestId::MIN;
        let moved_id = TestId::MAX;

        map.insert(removed_id, "removed");
        map.insert(moved_id, "moved");

        assert_eq!(map.remove(removed_id), Some((removed_id, "removed")));
        assert_eq!(map.get(moved_id), Some(&"moved"));
        assert_eq!(map.remove(moved_id), Some((moved_id, "moved")));
        assert!(map.is_empty());
    }

    #[test]
    fn clear_preserves_consistency_when_value_drop_unwinds() {
        let mut map: IdMap<TestId8, PanicOnDrop> = IdMap::new();
        let id = TestId::MIN;

        map.insert(id, PanicOnDrop(true));

        let result = catch_unwind(AssertUnwindSafe(|| map.clear()));

        assert!(result.is_err());
        assert_eq!(map.len(), 0);
        assert!(!map.contains_key(id));
        assert!(map.get(id).is_none());

        map.insert(id, PanicOnDrop(false));

        assert_eq!(map.len(), 1);
        assert!(map.get(id).is_some());
    }

    #[test]
    fn equality_ignores_storage_order() {
        let first_id = TestId::MIN;
        let second_id = TestId::MAX;
        let mut first: IdMap<TestId8, &str> = IdMap::new();
        let mut second: IdMap<TestId8, &str> = IdMap::new();

        first.insert(first_id, "first");
        first.insert(second_id, "second");
        second.insert(second_id, "second");
        second.insert(first_id, "first");

        assert_eq!(first, second);

        second.insert(first_id, "changed");

        assert_ne!(first, second);
    }

    #[test]
    fn insert_hash_panic_does_not_append_storage() {
        let first_id = PanicHashId(TestId::MIN);
        let second_id = PanicHashId(TestId::MAX);
        let target_id = NonZeroed::<u8>::new(2).map(|value| PanicHashId(TestId8::raw(value)));
        let mut map: IdMap<PanicHashId, &str> = IdMap::new();

        assert!(target_id.is_some());

        let target_id = target_id.unwrap_or(first_id);

        map.insert(first_id, "first");
        map.insert(second_id, "second");

        HASH_CALLS.store(0, Ordering::SeqCst);
        PANIC_HASH_CALL.store(1, Ordering::SeqCst);

        let result = catch_unwind(AssertUnwindSafe(|| map.insert(target_id, "target")));

        PANIC_HASH_CALL.store(usize::MAX, Ordering::SeqCst);

        assert!(result.is_err());
        assert_eq!(HASH_CALLS.load(Ordering::SeqCst), 2);
        assert_eq!(map.len(), 2);
        assert_eq!(map.get(first_id), Some(&"first"));
        assert_eq!(map.get(second_id), Some(&"second"));
        assert!(!map.contains_key(target_id));
    }
}
