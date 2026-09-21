//! Token streams over statically borrowed storage.
//!
//! [`Store`] is the `'static` counterpart to a borrowed slice stream. It supports
//! infallible consumption, fixed lookahead, and cheap rewinding while retaining
//! one immutable backing token slice.

use core::convert::Infallible;

use super::{TokenStream, lookahead::Lookahead};

/// Rewindable infallible stream backed by a static token slice.
///
/// The position advances only after successful lookup and therefore never exceeds
/// storage length. [`Self::rewind`] creates an independent cursor over the same
/// immutable token data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Store<T>
where
    T: Copy + 'static,
{
    /// Backing token storage.
    storage: &'static [T],

    /// Zero-based position of the next token.
    storage_indice: usize,
}

// NOTE(invariant): `storage_indice` never exceeds `storage.len()` because consumption advances only after a successful lookup.
impl<T> Store<T>
where
    T: Copy + 'static,
{
    /// Construct a stream over static storage.
    #[inline]
    #[must_use]
    pub const fn new(storage: &'static [T]) -> Self {
        let storage_indice = usize::MIN;

        Self { storage, storage_indice }
    }

    /// Construct a fresh stream over the same storage.
    #[inline]
    #[must_use]
    pub const fn rewind(&self) -> Self {
        let &Self { storage, .. } = self;

        Self::new(storage)
    }

    /// Retrieve the backing storage.
    #[inline]
    #[must_use]
    pub const fn storage(&self) -> &'static [T] {
        let &Self { storage, .. } = self;

        storage
    }
}

impl<T> Lookahead for Store<T>
where
    T: Copy + 'static,
{
    #[inline]
    fn lookahead<const N: usize>(&self) -> Result<Option<Self::Token>, Self::Error> {
        let &Self { storage, storage_indice } = self;
        let token = storage.get(storage_indice.saturating_add(N)).copied();

        Ok(token)
    }
}

impl<T> TokenStream for Store<T>
where
    T: Copy + 'static,
{
    type Token = T;
    type Error = Infallible;

    #[inline]
    fn next(&mut self) -> Result<Option<Self::Token>, Self::Error> {
        let &mut Self {
            storage,
            ref mut storage_indice,
        } = self;

        let target_value = storage.get(*storage_indice).copied();

        *storage_indice += usize::from(target_value.is_some());

        Ok(target_value)
    }
}
