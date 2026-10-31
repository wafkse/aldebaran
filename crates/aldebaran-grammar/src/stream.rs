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

/// A stream that consumes parser tokens in source order.
pub trait TokenStream {
    /// Token type yielded by the stream.
    type Token;

    /// Failure produced while obtaining the next token.
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
