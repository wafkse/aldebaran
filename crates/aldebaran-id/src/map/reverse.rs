//! Positional access to key/value pairs without key lookup or uniqueness checks.
//!
//! [`ReverseMap`] exposes aligned key and value slices, so the same element or
//! range index selects corresponding parts of both views.

use core::{iter::Zip, mem, ops::IndexMut, slice::SliceIndex};

use alloc::vec::Vec;

/// Corresponding key and value selections, including aligned ranges of pairs.
type IndexedKeyValue<'a, K, V, I> = (&'a <I as SliceIndex<[K]>>::Output, &'a <I as SliceIndex<[V]>>::Output);

/// Preserve key/value associations across positional access and value mutation.
/// Duplicate keys are permitted, and removal may change the position of a surviving pair.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
// NOTE(invariant): Keys.Length = Values.Length And ForAll(Index In 0..Keys.Length): Entry(Index) = (Keys[Index], Values[Index])
pub struct ReverseMap<K, V> {
    /// Key view aligned by index and length with the value view.
    keys: Vec<K>,

    /// Value view aligned by index and length with the key view.
    values: Vec<V>,
}

impl<K, V> ReverseMap<K, V> {
    /// Create an empty pair sequence.
    #[inline]
    pub fn new() -> Self {
        let keys = Vec::new();
        let values = Vec::new();

        Self { keys, values }
    }
}

impl<K, V> ReverseMap<K, V> {
    /// Number of pairs that fit before either backing vector grows.
    #[inline]
    pub fn capacity(&self) -> usize {
        let &Self { ref keys, ref values, .. } = self;

        keys.capacity().min(values.capacity())
    }

    /// Length shared by the aligned key and value views.
    #[inline]
    pub const fn len(&self) -> usize {
        let &Self { ref keys, .. } = self;

        keys.len()
    }

    /// Whether both aligned views are empty.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Append a pair without checking whether its key is already present.
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

    /// Discard every association while retaining capacity for reuse.
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

    /// Select values by pair position rather than key equality.
    /// An invalid element or range index yields `None`.
    #[inline]
    pub fn get<I>(&self, index: I) -> Option<&<I as SliceIndex<[V]>>::Output>
    where
        I: SliceIndex<[V]>,
    {
        let &Self { ref values, .. } = self;

        values.get(index)
    }

    /// Edit values selected by pair position without changing their keys or order.
    /// An invalid element or range index yields `None`.
    #[inline]
    pub fn get_mut<I>(&mut self, index: I) -> Option<&mut <I as SliceIndex<[V]>>::Output>
    where
        I: SliceIndex<[V]>,
    {
        let &mut Self { ref mut values, .. } = self;

        values.get_mut(index)
    }

    /// Select keys by pair position, including duplicate keys.
    /// An invalid element or range index yields `None`.
    #[inline]
    pub fn get_key<I>(&self, index: I) -> Option<&<I as SliceIndex<[K]>>::Output>
    where
        I: SliceIndex<[K]>,
    {
        let &Self { ref keys, .. } = self;

        keys.get(index)
    }

    /// Select corresponding keys and values by pair position.
    /// Ranges preserve their alignment, and an invalid index yields `None`.
    #[inline]
    pub fn get_key_value<I>(&self, index: I) -> Option<IndexedKeyValue<'_, K, V, I>>
    where
        I: SliceIndex<[K]> + SliceIndex<[V]> + Copy,
    {
        let &Self { ref keys, ref values, .. } = self;

        keys.get(index).and_then(|k| values.get(index).map(|v| (k, v)))
    }

    /// Keys aligned by index with [`Self::values`], including duplicate keys.
    #[inline]
    pub fn keys(&self) -> &[K] {
        let &Self { ref keys, .. } = self;

        keys.as_slice()
    }

    /// Values aligned by index with [`Self::keys`], including after positional replacement
    /// or removal.
    #[inline]
    pub fn values(&self) -> &[V] {
        let &Self { ref values, .. } = self;

        values.as_slice()
    }

    /// Remove the pair at `index`, moving the final pair into its position.
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

    /// Replace the pair at `index`, returning the previous pair.
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

        let key = mem::replace(keys.index_mut(index), k);
        let value = mem::replace(values.index_mut(index), v);

        (key, value)
    }

    /// Traverse the associations represented by matching positions in the key
    /// and value views.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        let &Self { ref keys, ref values, .. } = self;

        keys.iter().zip(values.iter())
    }

    /// Edit values alongside their associated keys without changing pair order.
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

        keys.into_iter().zip(values)
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
    fn clear() {
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
    fn swap_remove_last() {
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
    fn swap_at_same_index() {
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
    fn extend() {
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
    fn push() {
        let mut map: ReverseMap<u32, u32> = ReverseMap::new();

        map.push(1, 10);
        map.push(2, 20);
        map.push(3, 30);

        assert_eq!(map.len(), 3);
        assert_eq!(map.keys(), &[1, 2, 3]);
        assert_eq!(map.values(), &[10, 20, 30]);
    }

    #[test]
    fn get() {
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
    fn get_mut() {
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
    fn get_key() {
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
    fn get_key_value() {
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
    fn iter() {
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
    fn iter_mut() {
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
