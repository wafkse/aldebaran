//! Borrowed nonempty collection views.
//!
//! [`RefOneOrMore`] represents one required first element followed by an optional
//! borrowed tail. The representation makes emptiness impossible while avoiding
//! allocation and supports iteration through the shared one-or-more iterator.

use core::num::NonZero;

use super::iter::OnceOrMore;

/// Borrowed view that guarantees at least one element is present.
///
/// The first element is stored separately from the remaining slice, making the
/// nonempty invariant intrinsic to the representation. This is the borrowed
/// counterpart to [`super::OneOrMore`].
#[derive(Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct RefOneOrMore<'a, T> {
    pub(crate) a0: &'a T,
    pub(crate) a_n: &'a [T],
}

impl<'a, T> Copy for RefOneOrMore<'a, T> {}

impl<'a, T> Clone for RefOneOrMore<'a, T> {
    #[inline]
    fn clone(&self) -> Self {
        let &Self { a0, a_n } = self;

        Self { a0, a_n }
    }
}

impl<'a, T> RefOneOrMore<'a, T> {
    /// Create a new [`RefOneOrMore`] with the specified first value and the rest
    /// of the values.
    #[inline]
    pub const fn new(a0: &'a T, a_n: &'a [T]) -> Self {
        Self { a0, a_n }
    }

    /// Create a new [`RefOneOrMore`] with a single value.
    #[inline]
    pub const fn single(a0: &'a T) -> Self {
        Self { a0, a_n: &[] }
    }

    /// Create a new [`RefOneOrMore`] from a 2-element tuple representing a
    /// first + slice pair.
    #[inline]
    pub const fn tuple((a0, a_n): (&'a T, &'a [T])) -> Self {
        Self { a0, a_n }
    }

    /// Retrieve the first value.
    #[inline]
    pub const fn first(&self) -> &'a T {
        let &Self { a0, .. } = self;

        a0
    }

    /// Retrieve the rest of the values, that is, all values except the first
    /// one.
    #[inline]
    pub const fn rest(&self) -> &'a [T] {
        let &Self { a_n, .. } = self;

        a_n
    }

    /// Retrieve the raw parts of this [`RefOneOrMore`].
    #[inline]
    pub const fn into_raw_parts(self) -> (&'a T, &'a [T]) {
        let Self { a0, a_n } = self;

        (a0, a_n)
    }

    /// Retrieve the `nth` value.
    #[inline]
    pub fn nth(&self, n: NonZero<usize>) -> Option<&'a T> {
        let &Self { a_n, .. } = self;

        let target_index = n.get() - 1; // REMARK: will never underflow, as `NonZero` is always at least 1.

        a_n.get(target_index)
    }

    /// Determine the length of this [`RefOneOrMore`].
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
