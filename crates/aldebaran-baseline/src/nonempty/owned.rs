//! Owned collections that guarantee at least one element.

use core::{
    iter::{self, FusedIterator},
    num::NonZero,
};

use alloc::vec::Vec;

use super::RefNonempty;

/// A structure that guarantees that there is at least one known element.
///
/// For a reference-based version, see [`RefNonempty`]. The required first value
/// is stored separately from the owned tail, and iteration visits them in order.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
// NOTE(invariant): None.
pub struct Nonempty<T>((T, Vec<T>));

impl<T> Nonempty<T> {
    /// Create a new [`Nonempty`] with a single value.
    #[inline]
    pub const fn single(target_value: T) -> Self {
        Self((target_value, Vec::new()))
    }

    /// Create a new [`Nonempty`] with the specified first value and the rest.
    #[inline]
    pub const fn multiple(target_single: T, target_list: Vec<T>) -> Self {
        Self((target_single, target_list))
    }

    /// Attempt to create a new [`Nonempty`] collection from a [`Vec`].
    ///
    /// Returns [`None`] if the vector is empty. The vector's element order is
    /// preserved.
    #[inline]
    pub fn vector(values: Vec<T>) -> Option<Self> {
        let mut values = values.into_iter();

        match values.next() {
            Some(first) => {
                let rest = values.collect();

                Some(Self::multiple(first, rest))
            }
            None => None,
        }
    }

    /// Retrieve the first value and the rest of the values as a pair.
    #[inline]
    pub const fn tuple(&self) -> &(T, Vec<T>) {
        let Self(target_tuple) = self;

        target_tuple
    }

    /// Count the required value and tail, saturating at `usize::MAX`.
    #[inline]
    pub const fn len(&self) -> NonZero<usize> {
        let Self((.., target_list)) = self;

        NonZero::<usize>::MIN.saturating_add(target_list.len())
    }

    /// Retrieve a reference to the first value contained in this [`Nonempty`].
    #[inline]
    pub const fn first(&self) -> &T {
        let Self((target_value, ..)) = self;

        target_value
    }

    /// Retrieve a mutable reference to the first value contained in this
    /// [`Nonempty`].
    #[inline]
    pub const fn first_mut(&mut self) -> &mut T {
        let Self((target_value, ..)) = self;

        target_value
    }

    /// Retrieve a slice of the values following the first one.
    #[inline]
    pub const fn rest(&self) -> &[T] {
        let Self((.., target_list)) = self;

        target_list.as_slice()
    }

    /// Retrieve a mutable slice of the values following the first one. Its length
    /// cannot be changed through this slice.
    #[inline]
    pub const fn rest_mut(&mut self) -> &mut [T] {
        let Self((.., target_list)) = self;

        target_list.as_mut_slice()
    }

    /// The growable tail, excluding the required value.
    #[inline]
    pub const fn list(&self) -> &Vec<T> {
        let Self((.., target_list)) = self;

        target_list
    }

    /// Insert or remove tail elements while the required value remains present.
    #[inline]
    pub const fn list_mut(&mut self) -> &mut Vec<T> {
        let Self((.., target_list)) = self;

        target_list
    }

    /// Retrieve an iterator over all values.
    #[inline]
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &T> + FusedIterator + use<'_, T> {
        let Self((target_value, target_list)) = self;

        iter::once(target_value).chain(target_list.iter())
    }

    /// Retrieve an iterator over all values, mutably.
    #[inline]
    pub fn iter_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut T> + FusedIterator + use<'_, T> {
        let Self((target_value, target_list)) = self;

        iter::once(target_value).chain(target_list.iter_mut())
    }

    /// Convert this [`Nonempty`] into its referential counterpart
    /// [`RefNonempty`].
    #[inline]
    pub const fn as_ref(&self) -> RefNonempty<'_, T> {
        let Self((first, rest)) = self;

        let rest = rest.as_slice();

        RefNonempty::multiple(first, rest)
    }
}

#[cfg(test)]
mod tests {
    //! Tests module for the [`owned`] module.
    //!
    //! [`owned`]: super

    use alloc::{string::String, vec, vec::Vec};
    use core::{num::NonZero, ptr};

    use super::Nonempty;

    #[test]
    fn vector_conversion_preserves_empty_singleton_and_ordered_inputs() {
        assert_eq!(Nonempty::<u32>::vector(Vec::new()), None);
        assert_eq!(Nonempty::vector(vec![7]), Some(Nonempty::single(7)));

        let values = Nonempty::vector(vec![1, 2, 3, 4]).expect("nonempty vector");

        assert_eq!(*values.first(), 1);
        assert_eq!(values.rest(), [2, 3, 4]);
        assert_eq!(values.iter().copied().collect::<Vec<_>>(), [1, 2, 3, 4]);
    }

    #[test]
    fn tail_growth_and_removal_retain_the_required_value() {
        let mut values = Nonempty::single(1);

        values.list_mut().extend([2, 3]);
        *values.first_mut() = 4;
        values.rest_mut().reverse();

        for value in values.iter_mut() {
            *value += 10;
        }

        assert_eq!(values.iter().copied().collect::<Vec<_>>(), [14, 13, 12]);

        values.list_mut().clear();

        assert_eq!(values.len(), NonZero::<usize>::MIN);
        assert_eq!(values.iter().copied().collect::<Vec<_>>(), [14]);
    }

    #[test]
    fn borrowed_views_share_owned_elements_and_preserve_order() {
        let first = String::from("primary");
        let rest = vec![String::from("related"), String::from("last")];
        let values = Nonempty::multiple(first, rest);
        let view = values.as_ref();

        assert!(ptr::eq(view.first(), values.first()), "the view must share the required value");
        assert!(ptr::eq(view.rest(), values.rest()), "the view must share the tail");
        assert_eq!(view.len(), NonZero::new(3).expect("three elements"));
        assert_eq!(view.iter().map(String::as_str).collect::<Vec<_>>(), ["primary", "related", "last"]);
    }
}
