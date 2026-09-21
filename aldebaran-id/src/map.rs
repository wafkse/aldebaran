//! Maps of [`trait@Id`]s to values.

use core::hash::BuildHasher;
use core::hash::Hash;

use aldebaran_dsa::prelude::HashMap;
use aldebaran_dsa::prelude::ReverseMap;
use aldebaran_hash::fxhash::FxBuildHasher;

use crate::prelude::Id;

/// A map from [`trait@Id`]s to values.
///
/// # Time Complexity
///
/// The time complexity of the operations on this type are the same as those of
/// the underlying [`HashMap`], but:
///
/// - The [`trait@Id`]s are hashed using the [`core::hash::Hash::hash`] implementation, which is a no-op as identifiers are integers for the most part.
/// - The [`trait@Id`]s are compared using the [`core::cmp::PartialEq::eq`] implementation, which is a simple equality check.
///
/// So, the time complexity to access a value by its [`trait@Id`] is `O(1)`, in all
/// cases be it amortized or worst-case.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdMap<I, V, S = FxBuildHasher>
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
    /// # Time Complexity
    ///
    /// The time complexity of this operation is `O(2K)`, where `K` is the time
    /// complexity of creating two allocators of type `A`.
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
        // fixme: make fn const after this stabilizes
        let map = HashMap::with_hasher(hasher);
        let storage = ReverseMap::new();

        Self { map, storage }
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
    #[inline]
    pub fn capacity(&self) -> usize {
        let &Self { ref storage, ref map, .. } = self;

        // FIXME: make this a const fn after this stabilizes
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

        // FIXME: make this a const fn after this stabilizes
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
    /// # Time Complexity
    ///
    /// The time complexity of this operation is `O(n)`.
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

    /// Determine if the target [`trait@Id`] is contained in this [`IdMap`].
    ///
    /// # Time Complexity
    ///
    /// The time complexity of this operation is `O(1)`.
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
    /// The time complexity of this operation is `O(1)`.
    #[inline]
    pub fn keys<'a>(&'a self) -> impl Iterator<Item = &'a I> + 'a {
        let &Self { ref map, .. } = self;

        map.keys()
    }

    /// Retrieve an iterator over the values of entries of this [`IdMap`].
    ///
    /// # Time Complexity
    ///
    /// The time complexity of this operation is `O(1)`.
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
    /// The time complexity of this operation is `O(1)`.
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
    /// The time complexity of this operation is `O(1)`.
    #[inline]
    pub fn as_values(&self) -> &[V] {
        let &Self { ref storage, .. } = self;

        // FIXME: make this a const fn after this stabilizes
        storage.values()
    }

    /// Retrieve a reference to a slice of the keys of the entries of this
    /// [`IdMap`].
    ///
    /// # Time Complexity
    ///
    /// The time complexity of this operation is `O(1)`.
    #[inline]
    pub fn as_keys(&self) -> &[I] {
        let &Self { ref storage, .. } = self;

        // FIXME: make this a const fn after this stabilizes
        storage.keys()
    }

    /// Retrieve a reference to the value corresponding to the given [`trait@Id`].
    ///
    /// # Time Complexity
    ///
    /// The time complexity of this operation is `O(1)`.
    #[inline]
    pub fn get<Q>(&self, id: Q) -> Option<&V>
    where
        Q: AsRef<I>,
        Q: Eq + Hash,
    {
        let &Self { ref map, ref storage, .. } = self;

        map.get(id.as_ref()).map(|&index| storage.get(index)).flatten()
    }

    /// Retrieve a mutable reference to the value corresponding to the given
    /// [`trait@Id`] in this [`IdMap`].
    ///
    /// # Time Complexity
    ///
    /// The time complexity of this operation is `O(1)`.
    #[inline]
    pub fn get_mut<Q>(&mut self, id: &Q) -> Option<&mut V>
    where
        Q: AsRef<I>,
        Q: Eq + Hash + ?Sized,
    {
        let &mut Self {
            ref map, ref mut storage, ..
        } = self;

        map.get(id.as_ref()).map(move |&index| storage.get_mut(index)).flatten()
    }

    /// Insert a new entry into this [`IdMap`], potentially replacing an
    /// existing one.
    ///
    /// # Time Complexity
    ///
    /// The time complexity of this operation is `O(1)`. (amortized).
    #[inline]
    pub fn insert(&mut self, id: I, value: V) -> Option<V> {
        let &mut Self {
            ref mut map,
            ref mut storage,
            ..
        } = self;

        let target_index = map.get(&id).map(|&i| i);

        if let Some(target_index) = target_index {
            let (target_key, target_value) = storage.swap_at(target_index, (id, value));

            // Update the map to reflect the new index of the target key
            let _ = map.insert(target_key, target_index);

            Some(target_value)
        } else {
            let target_index = storage.len();

            storage.push(id, value);

            let _ = map.insert(id, target_index);

            None
        }
    }

    /// Remove the entry corresponding to the given [`trait@Id`] from this [`IdMap`].
    ///
    /// # Time Complexity
    ///
    /// The time complexity of this operation is `O(1)` (amortized).
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

        target_index
            .map(|target_index: usize| -> Option<(I, V)> {
                let (target_key, target_value) = storage.swap_remove(target_index);

                let target_id = storage.get_key(target_index).expect("index removed from storage");

                // Update the map to reflect the new index of the target key
                map.insert(*target_id, target_index);

                Some((target_key, target_value))
            })
            .flatten()
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
            let _ = self.insert(id, value);
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
    use super::*;

    use crate::Id;

    Id!(
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        become TestId
    );

    TestId!(8 become);

    #[test]
    fn test_id_map() {
        let mut map: IdMap<TestId8, &str, FxBuildHasher> = IdMap::new();

        assert_eq!(map.len(), 0);

        map.insert(TestId::MIN, "nignog");

        map.insert(TestId::MAX, "nignag");

        dbg!(map);
    }
}
