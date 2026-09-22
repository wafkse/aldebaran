//! General-purpose and Aldebaran-specific collection types.
//!
//! The module combines allocation-backed collection re-exports with structures
//! such as [`ReverseMap`] that support compiler-oriented indexing patterns.

pub mod one_or_more;

pub mod static_vec;

pub use alloc::vec::Vec;

pub use one_or_more::{OneOrMore, RefOneOrMore};

pub use smallvec::{Array, SmallVec};

pub use crate::alloc::collections::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque};

pub use hashbrown::{
    hash_map::{self, HashMap},
    hash_set::{self, HashSet},
};

use core::{iter::Zip, ops::IndexMut, slice::SliceIndex};

/// A map that has a reverse mapping from values to keys.
///
/// This is crucial for certain algorithms and data structures to work in `O(1)`
/// time complexity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]

pub struct ReverseMap<K, V> {
    /// The keys of the map.
    keys: Vec<K>,
    /// The values of the map.
    values: Vec<V>,
}

impl<K, V> ReverseMap<K, V> {
    /// Create a new empty [`ReverseMap`].
    #[inline]
    pub fn new() -> Self {
        let keys = Vec::new();
        let values = Vec::new();

        Self { keys, values }
    }
}

impl<K, V> ReverseMap<K, V> {
    /// Query the capacity of the [`ReverseMap`], that is, the maximum number of
    /// elements it can hold without reallocating.
    ///
    /// This operation is `O(1)`.
    #[inline]
    pub fn capacity(&self) -> usize {
        let &Self { ref keys, ref values, .. } = self;

        // FIXME: make this a const fn after this stabilizes
        keys.capacity().min(values.capacity())
    }

    /// Query the number of elements in the [`ReverseMap`].
    ///
    /// This operation is `O(1)`.
    #[inline]
    pub fn len(&self) -> usize {
        let &Self { ref keys, ref values, .. } = self;

        assert_eq!(keys.len(), values.len());

        // FIXME: make this a const fn after this stabilizes
        keys.len()
    }

    /// Query if the [`ReverseMap`] is empty.
    ///
    /// This operation is `O(1)`.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Push a key-value pair into the [`ReverseMap`].
    ///
    /// This operation is `O(1)` (amortized). In the worst case it is `O(n +
    /// 1)`, where `n` is the number of elements in the map, plus the additional
    /// allocation cost of the underlying allocator.
    #[inline]
    pub fn push(&mut self, key: K, value: V) {
        let &mut Self {
            ref mut keys,
            ref mut values,
            ..
        } = self;

        keys.push(key);
        values.push(value);
    }

    /// Clear the [`ReverseMap`], removing all elements.
    ///
    /// This operation is `O(n)` worst-case.
    #[inline]
    pub fn clear(&mut self) {
        let &mut Self {
            ref mut keys,
            ref mut values,
            ..
        } = self;

        keys.clear();
        values.clear();
    }

    /// Retrieve the value `V` from the [`ReverseMap`] by index.
    ///
    /// Time complexity: `O(1)`.
    #[inline]
    pub fn get<I>(&self, index: I) -> Option<&<I as SliceIndex<[V]>>::Output>
    where
        I: SliceIndex<[V]>,
    {
        let &Self { ref values, .. } = self;

        values.get(index)
    }

    /// Retrieve the value `V` from the [`ReverseMap`] by index, mutably.
    ///
    /// Time complexity: `O(1)`.
    #[inline]
    pub fn get_mut<I>(&mut self, index: I) -> Option<&mut <I as SliceIndex<[V]>>::Output>
    where
        I: SliceIndex<[V]>,
    {
        let &mut Self { ref mut values, .. } = self;

        values.get_mut(index)
    }

    /// Retrieve the key `K` from the [`ReverseMap`] by index.
    ///
    /// Time complexity: `O(1)`.
    #[inline]
    pub fn get_key<I>(&self, index: I) -> Option<&<I as SliceIndex<[K]>>::Output>
    where
        I: SliceIndex<[K]>,
    {
        let &Self { ref keys, .. } = self;

        keys.get(index)
    }

    /// Retrieve one or many key-value pair from the [`ReverseMap`] by index.
    ///
    /// Time complexity: `O(1)` with a constant factor of `2`.
    #[inline]
    pub fn get_key_value<I>(&self, index: I) -> Option<(&<I as SliceIndex<[K]>>::Output, &<I as SliceIndex<[V]>>::Output)>
    where
        I: SliceIndex<[K]> + SliceIndex<[V]> + Copy,
    {
        let &Self { ref keys, ref values, .. } = self;

        keys.get(index).and_then(|k| values.get(index).map(|v| (k, v)))
    }

    /// Retrieve a slice of keys from the [`ReverseMap`].
    ///
    /// Time complexity: `O(1)`.
    #[inline]
    pub fn keys(&self) -> &[K] {
        let &Self { ref keys, .. } = self;

        // FIXME: make this a const fn after this stabilizes
        keys.as_slice()
    }

    /// Retrieve a slice of values from the [`ReverseMap`].
    ///
    /// Time complexity: `O(1)`.
    #[inline]
    pub fn values(&self) -> &[V] {
        let &Self { ref values, .. } = self;

        // FIXME: make this a const fn after this stabilizes
        values.as_slice()
    }

    /// Perform a swap-remove operation on the [`ReverseMap`], that is, remove
    /// the key-value pair at the given index and swap it with the last element.
    ///
    /// This operation is `O(1)`.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    #[inline]
    pub fn swap_remove(&mut self, index: usize) -> (K, V) {
        let &mut Self {
            ref mut keys,
            ref mut values,
            ..
        } = self;

        let key = keys.swap_remove(index);
        let value = values.swap_remove(index);

        (key, value)
    }

    /// Swap the key-value pair at the given index with the given key-value
    /// pair.
    ///
    /// This operation is `O(1)`.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    #[inline]
    pub fn swap_at(&mut self, index: usize, kv: (K, V)) -> (K, V) {
        let &mut Self {
            ref mut keys,
            ref mut values,
            ..
        } = self;

        let (k, v) = kv;

        let key = core::mem::replace(keys.index_mut(index), k);
        let value = core::mem::replace(values.index_mut(index), v);

        (key, value)
    }

    /// Iterate over the key-value pairs of the [`ReverseMap`].
    ///
    /// Creating an iterator over the key-value pairs of the [`ReverseMap`] is
    /// `O(1)`, with no extra allocations.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        let &Self { ref keys, ref values, .. } = self;

        keys.iter().zip(values.iter())
    }

    /// Iterate over the key-value pairs of the [`ReverseMap`] mutably.
    ///
    /// Creating an iterator over the key-value pairs of the [`ReverseMap`] is
    /// `O(1)`, with no extra allocations.
    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&K, &mut V)> {
        let &mut Self {
            ref keys, ref mut values, ..
        } = self;

        keys.iter().zip(values.iter_mut())
    }
}

impl<K, V> Default for ReverseMap<K, V> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> IntoIterator for ReverseMap<K, V> {
    type Item = (K, V);

    type IntoIter = Zip<<Vec<K> as IntoIterator>::IntoIter, <Vec<V> as IntoIterator>::IntoIter>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { keys, values, .. } = self;

        keys.into_iter().zip(values.into_iter())
    }
}

impl<K, V> Extend<(K, V)> for ReverseMap<K, V> {
    #[inline]
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (K, V)>,
    {
        let &mut Self {
            ref mut keys,
            ref mut values,
            ..
        } = self;

        for (key, value) in iter {
            keys.push(key);
            values.push(value);
        }
    }
}

#[cfg(test)]
mod tests {

    use alloc::vec;

    use super::ReverseMap;

    #[test]
    fn test_clear() {
        let mut map: ReverseMap<u32, u32> = ReverseMap::new();
        map.push(1, 10);
        map.push(2, 20);
        map.push(3, 30);

        map.clear();

        assert_eq!(map.len(), 0);
        assert_eq!(map.keys(), &[]);
        assert_eq!(map.values(), &[]);
    }

    #[test]
    fn test_swap_remove_last() {
        let mut map: ReverseMap<u32, u32> = ReverseMap::new();
        map.push(1, 10);
        map.push(2, 20);
        map.push(3, 30);

        assert_eq!(map.swap_remove(2), (3, 30));
        assert_eq!(map.len(), 2);
        assert_eq!(map.keys(), &[1, 2]);
        assert_eq!(map.values(), &[10, 20]);
    }

    #[test]
    fn test_swap_at_same_index() {
        let mut map: ReverseMap<u32, u32> = ReverseMap::new();
        map.push(1, 10);
        map.push(2, 20);
        map.push(3, 30);

        assert_eq!(map.swap_at(1, (2, 20)), (2, 20));
        assert_eq!(map.len(), 3);
        assert_eq!(map.keys(), &[1, 2, 3]);
        assert_eq!(map.values(), &[10, 20, 30]);
    }

    #[test]
    fn test_extend() {
        let mut map: ReverseMap<u32, u32> = ReverseMap::new();
        map.push(1, 10);
        map.push(2, 20);

        let additional_pairs = vec![(3, 30), (4, 40)];
        map.extend(additional_pairs);

        assert_eq!(map.len(), 4);
        assert_eq!(map.keys(), &[1, 2, 3, 4]);
        assert_eq!(map.values(), &[10, 20, 30, 40]);
    }

    #[test]
    fn test_push() {
        let mut map: ReverseMap<u32, u32> = ReverseMap::new();
        map.push(1, 10);
        map.push(2, 20);
        map.push(3, 30);

        assert_eq!(map.len(), 3);
        assert_eq!(map.keys(), &[1, 2, 3]);
        assert_eq!(map.values(), &[10, 20, 30]);
    }

    #[test]
    fn test_get() {
        let mut map: ReverseMap<u32, u32> = ReverseMap::new();
        map.push(1, 10);
        map.push(2, 20);
        map.push(3, 30);

        assert_eq!(map.get(0), Some(&10));
        assert_eq!(map.get(1), Some(&20));
        assert_eq!(map.get(2), Some(&30));
        assert_eq!(map.get(3), None);
    }

    #[test]
    fn test_get_mut() {
        let mut map: ReverseMap<u32, u32> = ReverseMap::new();
        map.push(1, 10);
        map.push(2, 20);
        map.push(3, 30);

        if let Some(value) = map.get_mut(1) {
            *value = 25;
        }

        assert_eq!(map.get(1), Some(&25));
    }

    #[test]
    fn test_get_key() {
        let mut map: ReverseMap<u32, u32> = ReverseMap::new();
        map.push(1, 10);
        map.push(2, 20);
        map.push(3, 30);

        assert_eq!(map.get_key(0), Some(&1));
        assert_eq!(map.get_key(1), Some(&2));
        assert_eq!(map.get_key(2), Some(&3));
        assert_eq!(map.get_key(3), None);
    }

    #[test]
    fn test_get_key_value() {
        let mut map: ReverseMap<u32, u32> = ReverseMap::new();
        map.push(1, 10);
        map.push(2, 20);
        map.push(3, 30);

        assert_eq!(map.get_key_value(0), Some((&1, &10)));
        assert_eq!(map.get_key_value(1), Some((&2, &20)));
        assert_eq!(map.get_key_value(2), Some((&3, &30)));
        assert_eq!(map.get_key_value(3), None);
    }

    #[test]
    fn test_iter() {
        let mut map: ReverseMap<u32, u32> = ReverseMap::new();
        map.push(1, 10);
        map.push(2, 20);
        map.push(3, 30);

        let mut iter = map.iter();
        assert_eq!(iter.next(), Some((&1, &10)));
        assert_eq!(iter.next(), Some((&2, &20)));
        assert_eq!(iter.next(), Some((&3, &30)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_iter_mut() {
        let mut map: ReverseMap<u32, u32> = ReverseMap::new();
        map.push(1, 10);
        map.push(2, 20);
        map.push(3, 30);

        for (_, value) in map.iter_mut() {
            *value += 5;
        }

        assert_eq!(map.get(0), Some(&15));
        assert_eq!(map.get(1), Some(&25));
        assert_eq!(map.get(2), Some(&35));
    }
}
