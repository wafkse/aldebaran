//! Token streams over statically borrowed storage.
//!
//! See [`Store`] for further information.

use core::convert::Infallible;

use super::{TokenStream, lookahead::Lookahead};

/// Rewindable infallible stream backed by a static token slice.
///
/// Clones and [`Self::rewind`] create independent cursors over the same token
/// table. The position advances only after successful lookup and never exceeds
/// storage length.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// NOTE(invariant): StorageIndex <= Storage.Length
pub struct Store<T>
where
    T: Copy + 'static,
{
    /// Backing token storage.
    storage: &'static [T],

    /// Zero-based position of the next token, or the storage length at exhaustion.
    storage_indice: usize,
}

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

    /// Create an independent stream at the start of the same table.
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

#[cfg(test)]
mod tests {
    //! Tests module for the [`storage`] module.
    //!
    //! [`storage`]: super

    use core::ptr;

    use super::Store;
    use crate::stream::{TokenStream, lookahead::Lookahead};

    #[test]
    fn rewind_restarts_without_resetting_the_original() {
        let mut stream = Store::new(&[3_u8, 5]);

        assert_eq!(stream.next(), Ok(Some(3)));

        let mut rewound = stream.rewind();

        assert!(
            ptr::eq(stream.storage(), rewound.storage()),
            "rewinding must retain the backing slice"
        );
        assert_eq!(rewound.next(), Ok(Some(3)));
        assert_eq!(stream.next(), Ok(Some(5)));
        assert_eq!(stream.next(), Ok(None));
        assert_eq!(rewound.next(), Ok(Some(5)));
    }

    #[test]
    fn lookahead_handles_overflow_and_repeated_exhaustion() {
        let mut stream = Store::new(&[3_u8, 5]);

        assert_eq!(stream.next(), Ok(Some(3)));
        assert_eq!(stream.lookahead::<0>(), Ok(Some(5)));
        assert_eq!(stream.lookahead::<{ usize::MAX }>(), Ok(None));
        assert_eq!(stream.next(), Ok(Some(5)));

        for _ in 0..3 {
            assert_eq!(stream.next(), Ok(None));
            assert_eq!(stream.lookahead::<0>(), Ok(None));
        }

        assert_eq!(stream.rewind().next(), Ok(Some(3)));
    }

    #[test]
    fn empty_storage_can_be_exhausted_and_rewound() {
        let mut stream = Store::<u8>::new(&[]);

        assert_eq!(stream.next(), Ok(None));
        assert_eq!(stream.next(), Ok(None));
        assert_eq!(stream.lookahead::<{ usize::MAX }>(), Ok(None));
        assert_eq!(stream.rewind().next(), Ok(None));
    }
}
