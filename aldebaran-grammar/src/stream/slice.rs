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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Slice<'a, T> {
    /// Borrowed token storage.
    storage: &'a [T],

    /// Zero-based position of the next token.
    index: usize,
}

impl<'a, T> Slice<'a, T> {
    /// Construct a token slice stream.
    #[inline]
    #[must_use]
    pub const fn new(storage: &'a [T]) -> Self {
        Self { storage, index: 0 }
    }
}

impl<T> Lookahead for Slice<'_, T>
where
    T: Copy,
{
    #[inline]
    fn lookahead<const N: usize>(&self) -> Result<Option<Self::Token>, Self::Error> {
        let &Self { storage, index } = self;
        let token = storage.get(index.saturating_add(N)).copied();

        Ok(token)
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
        let &mut Self { storage, ref mut index } = self;
        let token = storage.get(*index).copied();

        *index += usize::from(token.is_some());

        Ok(token)
    }
}
