//! Infallible access to a borrowed token sequence.
//!
//! [`Slice`] keeps a position within that sequence, allowing copied streams to
//! advance independently while borrowing the same tokens.

use core::convert::Infallible;

use super::{TokenStream, lookahead::Lookahead};

/// Infallible stream over a borrowed slice of copyable tokens.
///
/// Reads return tokens by value. Repeated reads after exhaustion leave the
/// position at the slice length.
#[derive(Debug, PartialEq, Eq, Hash)]
// NOTE(invariant): The stored position never exceeds the length of the borrowed token slice.
pub struct Slice<'a, T>(&'a [T], usize);

impl<T> Clone for Slice<'_, T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Slice<'_, T> {}

impl<'a, T> Slice<'a, T> {
    /// Create a stream at the start of `storage`.
    #[inline]
    #[must_use]
    pub const fn new(storage: &'a [T]) -> Self {
        Self(storage, usize::MIN)
    }

    /// Zero-based offset of the next unread token in the original slice.
    /// The slice length marks exhaustion and remains unchanged on empty reads.
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

#[cfg(test)]
mod tests {
    use super::Slice;
    use crate::stream::{TokenStream, lookahead::Lookahead};

    #[test]
    fn exhaustion_preserves_the_terminal_position() {
        let mut stream = Slice::new(&['a', 'b']);

        assert_eq!(stream.next(), Ok(Some('a')));
        assert_eq!(stream.position(), 1);
        assert_eq!(stream.next(), Ok(Some('b')));

        for _ in 0..3 {
            assert_eq!(stream.next(), Ok(None));
            assert_eq!(stream.lookahead::<0>(), Ok(None));
            assert_eq!(stream.position(), 2);
        }
    }

    #[test]
    fn empty_storage_remains_at_its_origin() {
        let mut stream = Slice::<u8>::new(&[]);

        assert_eq!(stream.lookahead::<0>(), Ok(None));
        assert_eq!(stream.next(), Ok(None));
        assert_eq!(stream.next(), Ok(None));
        assert_eq!(stream.position(), 0);
    }

    #[test]
    fn lookahead_handles_overflow_without_advancing() {
        let mut stream = Slice::new(&[3_u8, 5, 7]);

        assert_eq!(stream.next(), Ok(Some(3)));
        assert_eq!(stream.lookahead::<1>(), Ok(Some(7)));
        assert_eq!(stream.lookahead::<2>(), Ok(None));
        assert_eq!(stream.lookahead::<{ usize::MAX }>(), Ok(None));
        assert_eq!(stream.position(), 1);
        assert_eq!(stream.next(), Ok(Some(5)));
    }

    #[test]
    fn copied_cursors_consume_independently() {
        let mut stream = Slice::new(&[3_u8, 5, 7]);

        assert_eq!(stream.next(), Ok(Some(3)));

        let mut copied = stream;

        assert_eq!(copied.next(), Ok(Some(5)));
        assert_eq!(copied.next(), Ok(Some(7)));
        assert_eq!(stream.position(), 1);
        assert_eq!(stream.next(), Ok(Some(5)));
    }
}
