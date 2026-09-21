//! Incremental token consumption and bounded lookahead abstractions.
//!
//! [`TokenStream`] represents only one operation, consuming the next token or
//! reporting exhaustion as `Ok(None)`. Wrappers add lookahead, filtering, borrowed
//! slices, and static storage without introducing a parser cursor protocol.

pub mod lookahead;
pub mod skipping;
pub mod slice;
pub mod storage;

/// A stream that consumes parser tokens in source order.
///
/// Exhaustion is represented only by `Ok(None)` from [`Self::next`].
/// There is no separate EOF token or stream state.
pub trait TokenStream {
    /// Token type yielded by the stream.
    type Token;

    /// Failure produced while obtaining the next token.
    type Error;

    /// Consume the next token.
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
