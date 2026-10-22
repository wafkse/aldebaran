//! Shared nonempty sequences for APIs that borrow their input collections.
//!
//! [`RefNonempty`] can borrow separate first and tail values or view an owned
//! [`Nonempty`](super::Nonempty). Copying the view shares the same elements and
//! does not require those elements to implement [`Copy`].

use core::{
    iter::{self, FusedIterator},
    num::NonZero,
};

/// A nonempty sequence borrowed without copying its elements or allocating.
///
/// Iteration visits the first value before the tail, and returned references
/// retain the input lifetime so they can outlive the view itself.
#[derive(Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
// NOTE(invariant): None.
pub struct RefNonempty<'a, T>((&'a T, &'a [T]));

impl<T> Copy for RefNonempty<'_, T> {}

impl<T> Clone for RefNonempty<'_, T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> RefNonempty<'a, T> {
    /// View a required value with no following elements.
    #[inline]
    pub const fn single(first: &'a T) -> Self {
        Self((first, &[]))
    }

    /// Place the required value before a borrowed tail without reordering it.
    #[inline]
    pub const fn multiple(first: &'a T, rest: &'a [T]) -> Self {
        Self((first, rest))
    }

    /// Accept a required value and borrowed tail supplied as a pair.
    #[inline]
    pub const fn tuple(parts: (&'a T, &'a [T])) -> Self {
        Self(parts)
    }

    /// The value that keeps the view nonempty even when the tail is empty.
    #[inline]
    pub const fn first(&self) -> &'a T {
        let Self((first, ..)) = *self;

        first
    }

    /// Elements following the required value, in iteration order.
    #[inline]
    pub const fn rest(&self) -> &'a [T] {
        let Self((.., rest)) = *self;

        rest
    }

    /// Recover the input references without shortening their lifetimes.
    #[inline]
    pub const fn partitioned(self) -> (&'a T, &'a [T]) {
        let Self(parts) = self;

        parts
    }

    /// Count the required value and tail, saturating at `usize::MAX`.
    #[inline]
    pub const fn len(&self) -> NonZero<usize> {
        let Self((.., rest)) = self;

        NonZero::<usize>::MIN.saturating_add(rest.len())
    }

    /// Visit the input elements in order without borrowing the view itself.
    #[inline]
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &'a T> + FusedIterator + use<'a, T> {
        let Self((first, rest)) = *self;

        iter::once(first).chain(rest.iter())
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;
    use core::ptr;

    use super::RefNonempty;

    #[test]
    fn iterators_survive_borrowed_views_and_remain_exhausted() {
        let first = 1;
        let rest = [2, 3];
        let mut values = {
            let view = RefNonempty::multiple(&first, &rest);

            view.iter()
        };

        assert_eq!(values.next_back(), Some(&3));
        assert_eq!(values.next(), Some(&1));
        assert_eq!(values.next_back(), Some(&2));
        assert_eq!(values.next(), None);
        assert_eq!(values.next_back(), None);
        assert_eq!(values.next(), None);
    }

    #[test]
    fn copying_a_borrowed_view_does_not_require_copy_elements() {
        /// A value that deliberately implements neither `Copy` nor `Clone`.
        #[derive(Debug, PartialEq, Eq)]
        // NOTE(invariant): None.
        struct Value(u32);

        let first = Value(1);
        let rest = [Value(2)];
        let view = RefNonempty::tuple((&first, &rest));
        let copy = view;

        assert!(
            ptr::eq(view.first(), copy.first()),
            "copying the view must retain its input references"
        );
        assert_eq!(copy.partitioned(), (&first, rest.as_slice()));
        assert_eq!(RefNonempty::single(&first).iter().collect::<Vec<_>>(), [&first]);
    }

    #[test]
    fn zero_sized_tail_lengths_saturate_without_losing_nonemptiness() {
        let first = ();
        let rest = [(); usize::MAX];
        let view = RefNonempty::multiple(&first, &rest);

        assert_eq!(view.len().get(), usize::MAX);
    }
}
