//! Incremental token consumption and lookahead abstractions.
//!
//! See [`TokenStream`] for further information.

pub mod lookahead;
pub mod skipping;
pub mod slice;
pub mod storage;

/// A stream that produces parser tokens in source order.
///
/// `Ok(None)` means that no token is available on this call, so a later call may
/// still return a token. Source failures are returned as `Err`.
pub trait TokenStream {
    /// Token type yielded by the stream.
    type Token;

    /// Failure produced while obtaining the next token.
    type Error;

    /// Consume the next token, if one is available on this read.
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
