//! Token streams over borrowed immutable slices.
//!
//! [`Slice`] retains a zero-based next-token position and provides infallible
//! consumption plus fixed lookahead. The cursor advances only when a token exists,
//! so repeated exhaustion remains stable without exceeding the backing slice.

use core::convert::Infallible;

use super::{TokenStream, lookahead::Lookahead};

/// Infallible token stream over one borrowed slice.
///
/// The stream never owns or copies the backing collection. Copyable tokens are
/// returned by value, and the internal index advances only after successful
/// lookup so exhaustion is repeatable.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Slice<'a, T>(&'a [T], usize);

impl<T> Clone for Slice<'_, T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Slice<'_, T> {}

impl<'a, T> Slice<'a, T> {
    /// Construct a token slice stream.
    #[inline]
    #[must_use]
    pub const fn new(storage: &'a [T]) -> Self {
        Self(storage, usize::MIN)
    }

    /// Return the zero-based position of the next token.
    ///
    /// The returned position is also the exclusive end position of every token
    /// already consumed by this cursor.
    #[inline]
    #[must_use]
    pub const fn position(&self) -> usize {
        let &Self(.., target_value) = self;

        target_value
    }
}

impl<T> Lookahead for Slice<'_, T>
where
    T: Copy,
{
    #[inline]
    fn lookahead<const N: usize>(&self) -> Result<Option<Self::Token>, Self::Error> {
        let &Self(target_storage, target_index) = self;

        Ok(target_storage.get(target_index.saturating_add(N)).copied())
    }
}

impl<T> TokenStream for Slice<'_, T>
where
    T: Copy,
{
    type Token = T;
    type Error = Infallible;

    #[inline]
    fn next(&mut self) -> Result<Option<Self::Token>, Self::Error> {
        let &mut Self(target_storage, ref mut target_index) = self;

        let target_value = target_storage.get(*target_index).copied();

        *target_index += usize::from(target_value.is_some());

        Ok(target_value)
    }
}
