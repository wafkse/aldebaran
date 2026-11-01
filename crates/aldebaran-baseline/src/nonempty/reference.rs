//! Borrowed nonempty collection views.
//!
//! See [`RefNonempty`] for further information.

use core::{
    iter::{self, FusedIterator},
    num::NonZero,
};

/// Borrowed view that guarantees at least one element is present.
///
/// The first element is stored separately from the remaining slice, making the
/// nonempty invariant intrinsic to the representation. This is the borrowed
/// counterpart to [`super::Nonempty`].
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
    /// Create a new [`RefNonempty`] with a single value.
    #[inline]
    pub const fn single(first: &'a T) -> Self {
        Self((first, &[]))
    }

    /// Create a new [`RefNonempty`] with the specified first value and the rest
    /// of the values.
    #[inline]
    pub const fn multiple(first: &'a T, rest: &'a [T]) -> Self {
        Self((first, rest))
    }

    /// Create a new [`RefNonempty`] from a 2-element tuple representing a
    /// first + slice pair.
    #[inline]
    pub const fn tuple(parts: (&'a T, &'a [T])) -> Self {
        Self(parts)
    }

    /// Retrieve the first value.
    #[inline]
    pub const fn first(&self) -> &'a T {
        let Self((first, ..)) = *self;

        first
    }

    /// Retrieve the rest of the values, that is, all values except the first
    /// one.
    #[inline]
    pub const fn rest(&self) -> &'a [T] {
        let Self((.., rest)) = *self;

        rest
    }

    /// Retrieve the raw parts of this [`RefNonempty`].
    ///
    /// The references retain their original lifetime.
    #[inline]
    pub const fn partitioned(self) -> (&'a T, &'a [T]) {
        let Self(parts) = self;

        parts
    }

    /// Determine the length of this [`RefNonempty`]. The result saturates at
    /// `usize::MAX` when the tail length is maximal.
    #[inline]
    pub const fn len(&self) -> NonZero<usize> {
        let Self((.., rest)) = self;

        NonZero::<usize>::MIN.saturating_add(rest.len())
    }

    /// Retrieve an iterator over all values. Its references retain the input
    /// lifetime, so the iterator may outlive this view.
    #[inline]
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &'a T> + FusedIterator + use<'a, T> {
        let Self((first, rest)) = *self;

        iter::once(first).chain(rest.iter())
    }
}

#[cfg(test)]
mod tests {
    //! Tests module for the [`reference`] module.
    //!
    //! [`reference`]: super

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
    fn zero_sized_tail_lengths_saturate_without_losing_nonemptiness() {
        let first = ();
        let rest = [(); usize::MAX];
        let view = RefNonempty::multiple(&first, &rest);

        assert_eq!(view.len().get(), usize::MAX);
    }
}
