//! ID lookup with position-aligned views of stored entries.
//!
//! [`IdMap`] associates each ID with a value, while [`reverse::ReverseMap`]
//! provides positional pair storage without requiring unique keys.

use core::hash::{BuildHasher, BuildHasherDefault, Hash};

pub mod reverse;

use crate::map::reverse::ReverseMap;
use hashbrown::HashMap;
use rustc_hash::FxHasher;

use crate::prelude::Id;

/// Associate values with IDs while retaining aligned key and value slice views.
/// New IDs append to those views, replacements retain their position, and removal
/// may reorder them independently of the hash-ordered [`Self::keys`] iterator.
#[derive(Debug, Clone, PartialEq, Eq)]
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

    /// Whether both paired views are empty and every ID lookup will fail.
    #[inline]
    pub fn is_empty(&self) -> bool {
        let &Self { ref storage, .. } = self;

        storage.is_empty()
    }

    /// Discard all associations while retaining capacity for reuse.
    #[inline]
    pub fn clear(&mut self) {
        let &mut Self {
            ref mut storage,
            ref mut map,
            ..
        } = self;

        storage.clear();
        map.clear();
    }

    /// Membership under ID equality, independent of the entry's storage position.
    #[inline]
    pub fn contains_key<Q>(&self, id: Q) -> bool
    where
        Q: AsRef<I>,
        Q: Eq + Hash,
    {
        let &Self { ref map, .. } = self;

        map.contains_key(id.as_ref())
    }

    /// Iterate over identifiers in unspecified hash-table order.
    #[inline]
    pub fn keys<'a>(&'a self) -> impl Iterator<Item = &'a I> + 'a {
        let &Self { ref map, .. } = self;

        map.keys()
    }

    /// Iterate over values in paired storage order, which may differ from
    /// [`Self::keys`].
    #[inline]
    pub fn values<'a>(&'a self) -> impl Iterator<Item = &'a V> + 'a {
        let &Self { ref storage, .. } = self;

        storage.values().iter()
    }

    /// Edit values in the order of [`Self::as_keys`] without changing their IDs
    /// or storage positions.
    #[inline]
    pub fn values_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut V> + 'a {
        let &mut Self { ref mut storage, .. } = self;

        storage.iter_mut().map(|(_, v)| v)
    }

    /// Values aligned by index with [`Self::as_keys`], including after reordering
    /// by removal.
    #[inline]
    pub fn as_values(&self) -> &[V] {
        let &Self { ref storage, .. } = self;

        storage.values()
    }

    /// IDs aligned by index with [`Self::as_values`], independently of the order
    /// produced by [`Self::keys`].
    #[inline]
    pub fn as_keys(&self) -> &[I] {
        let &Self { ref storage, .. } = self;

        storage.keys()
    }

    /// Resolve ID equality to its associated value, regardless of storage position.
    /// An absent ID yields `None`.
    #[inline]
    pub fn get<Q>(&self, id: Q) -> Option<&V>
    where
        Q: AsRef<I>,
        Q: Eq + Hash,
    {
        let &Self { ref map, ref storage, .. } = self;

        map.get(id.as_ref()).and_then(|&index| storage.get(index))
    }

    /// Edit an ID's associated value without changing its key or storage position.
    /// An absent ID yields `None`.
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

    /// Insert or replace an entry. New IDs append to the paired slices.
    /// Replacing an entry retains its position and returns the previous value.
    #[inline]
    pub fn insert(&mut self, id: I, value: V) -> Option<V> {
        let &mut Self {
            ref mut map,
            ref mut storage,
            ..
        } = self;

        let target_index = map.get(&id).copied();

        if let Some(target_index) = target_index {
            let (target_key, target_value) = storage.swap_at(target_index, (id, value));

            let _: Option<usize> = map.insert(target_key, target_index);

            Some(target_value)
        } else {
            let target_index = storage.len();

            storage.push(id, value);

            let _: Option<usize> = map.insert(id, target_index);

            None
        }
    }

    /// Remove an entry and return its stored identifier and value, if present.
    ///
    /// Removing an entry may move the final stored pair into its position.
    ///
    /// # Panics
    ///
    /// If the entry occupies the final storage position, this method panics
    /// after removing it. This includes removing the only entry.
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
            let (target_key, target_value) = storage.swap_remove(target_index);

            let target_id = storage.get_key(target_index).expect("index removed from storage");

            map.insert(*target_id, target_index);

            (target_key, target_value)
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

    Id!(
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        become TestId
    );

    TestId!(8 become);

    #[test]
    fn id_map() {
        let mut map: IdMap<TestId8, &str, BuildHasherDefault<FxHasher>> = IdMap::new();

        assert_eq!(map.len(), 0);

        map.insert(TestId::MIN, "nignog");

        map.insert(TestId::MAX, "nignag");

        drop(map);
    }
}
