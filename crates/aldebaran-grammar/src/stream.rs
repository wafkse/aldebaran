//! Pull-based token access for parsers.
//!
//! [`TokenStream`] keeps source errors distinct from an empty read, which need
//! not mark permanent exhaustion. [`lookahead`] inspects upcoming tokens,
//! [`skipping`] filters a stream, and [`mod@slice`] and [`storage`] provide
//! streams over token tables.

pub mod lookahead;
pub mod skipping;
pub mod slice;
pub mod storage;

/// Pull tokens while distinguishing source failures from unavailable input.
/// `Ok(None)` does not establish permanent exhaustion, and implementations define
/// whether a failing read advances the stream.
pub trait TokenStream {
    /// Unit of input consumed by parsers using this stream.
    type Token;

    /// Token-production failure, distinct from the absence of available input.
    type Error;

    /// Consume the next available token, preserving source failures as `Err`.
    /// An empty read does not establish permanent exhaustion.
    fn next(&mut self) -> Result<Option<Self::Token>, Self::Error>;
}

impl<S> TokenStream for &mut S
where
    S: TokenStream + ?Sized,
{
    type Token = S::Token;
    type Error = S::Error;

    #[inline]
    fn next(&mut self) -> Result<Option<Self::Token>, Self::Error> {
        S::next(self)
    }
}
