//! Token stream filtering that preserves the underlying stream.
//!
//! [`Skipping`] repeatedly consumes selected tokens while exposing only the
//! remaining tokens through [`TokenStream`]. Lookahead clones the filtered view
//! so inspection follows the same skip policy without advancing the caller.

use super::{TokenStream, lookahead::Lookahead};

/// A token stream view that hides tokens selected by one predicate.
///
/// Consumption repeatedly advances the underlying stream until a visible token
/// or source exhaustion is reached. Lookahead clones this complete view, which
/// guarantees that observation and consumption apply the same filtering policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Skipping<S, P> {
    /// Underlying token stream.
    stream: S,

    /// Predicate selecting tokens that must remain invisible to consumers.
    skip: P,
}

impl<S, P> Skipping<S, P> {
    /// Construct a skipping view over one token stream.
    #[inline]
    #[must_use]
    pub const fn new(stream: S, skip: P) -> Self {
        Self { stream, skip }
    }
}

impl<S, P> TokenStream for Skipping<S, P>
where
    S: TokenStream,
    P: Fn(&S::Token) -> bool,
{
    type Token = S::Token;
    type Error = S::Error;

    #[inline]
    fn next(&mut self) -> Result<Option<Self::Token>, Self::Error> {
        let Self { stream, skip } = self;

        loop {
            match S::next(stream)? {
                Some(token) => {
                    if !skip(&token) {
                        break Ok(Some(token));
                    }
                }
                None => break Ok(None),
            }
        }
    }
}

impl<S, P> Lookahead for Skipping<S, P>
where
    S: Lookahead + Clone,
    P: Fn(&S::Token) -> bool + Clone,
{
    #[inline]
    fn lookahead<const N: usize>(&self) -> Result<Option<Self::Token>, Self::Error> {
        let mut stream = self.clone();
        let mut remaining = N;

        loop {
            let token = TokenStream::next(&mut stream);

            match (remaining, token) {
                (_, Err(error)) => break Err(error),
                (_, Ok(None)) => break Ok(None),
                (0, Ok(Some(token))) => break Ok(Some(token)),
                (_, Ok(Some(_))) => remaining = remaining.saturating_sub(1),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::stream::{TokenStream, lookahead::Lookahead, slice::Slice};

    use super::Skipping;

    #[test]
    fn skips_selected_tokens_for_consumption_and_lookahead() {
        let tokens = [0u8, 1, 0, 2];
        let stream = Slice::new(&tokens);
        let mut stream = Skipping::new(stream, |token: &u8| *token == 0);

        assert_eq!(stream.lookahead::<0>(), Ok(Some(1)));
        assert_eq!(stream.lookahead::<1>(), Ok(Some(2)));
        assert_eq!(stream.next(), Ok(Some(1)));
        assert_eq!(stream.next(), Ok(Some(2)));
        assert_eq!(stream.next(), Ok(None));
    }
}
