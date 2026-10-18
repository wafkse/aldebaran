//! Borrowed views of nonempty sequences.

use core::num::NonZero;

use super::iter::OnceOrMore;

/// A borrowed nonempty sequence with access to its first value and tail.
///
/// The owned counterpart is [`super::OneOrMore`].
#[derive(Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
// NOTE(invariant): None.
pub struct RefOneOrMore<'a, T> {
    /// The required shared first element that guarantees a nonempty view.
    pub(in crate::nonempty) a0: &'a T,

    /// Shared elements following the required first element.
    pub(in crate::nonempty) a_n: &'a [T],
}

impl<'a, T> Copy for RefOneOrMore<'a, T> {}

impl<'a, T> Clone for RefOneOrMore<'a, T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> RefOneOrMore<'a, T> {
    /// Borrow a required value followed by a slice.
    #[inline]
    pub const fn new(a0: &'a T, a_n: &'a [T]) -> Self {
        Self { a0, a_n }
    }

    /// Borrow a sequence containing only the required value.
    #[inline]
    pub const fn single(a0: &'a T) -> Self {
        Self { a0, a_n: &[] }
    }

    /// Borrow a sequence from its required value and tail pair.
    #[inline]
    pub const fn tuple((a0, a_n): (&'a T, &'a [T])) -> Self {
        Self { a0, a_n }
    }

    /// Borrow the required value.
    #[inline]
    pub const fn first(&self) -> &'a T {
        let &Self { a0, .. } = self;

        a0
    }

    /// Borrow the tail after the required value.
    #[inline]
    pub const fn rest(&self) -> &'a [T] {
        let &Self { a_n, .. } = self;

        a_n
    }

    /// Consume the view into its required value and tail pair.
    #[inline]
    pub const fn into_raw_parts(self) -> (&'a T, &'a [T]) {
        let Self { a0, a_n } = self;

        (a0, a_n)
    }

    /// Index the tail using one-based numbering.
    ///
    /// Because the required first value is excluded, position `1` selects the
    /// first tail value, and positions beyond the tail return [`None`]. To include
    /// the required value in traversal, use [`Self::iter`] instead.
    #[inline]
    pub fn nth(&self, n: NonZero<usize>) -> Option<&'a T> {
        let &Self { a_n, .. } = self;

        let target_index = n.get() - 1; // REMARK: will never underflow, as `NonZero` is always at least 1.

        a_n.get(target_index)
    }

    /// Return the total number of elements, including the required value.
    #[inline]
    pub const fn len(&self) -> NonZero<usize> {
        let &Self { a_n, .. } = self;

        match NonZero::new(1 + a_n.len()) {
            Some(len) => len,
            None => unreachable!(),
        }
    }

    /// Retrieve an iterator over all values.
    #[inline]
    pub fn iter(&self) -> OnceOrMore<'a, T> {
        OnceOrMore::from_ref(*self)
    }
}
