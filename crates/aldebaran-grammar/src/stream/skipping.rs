//! Token filtering that preserves the source's error type.
//!
//! [`Skipping`] excludes selected tokens from reads and lookahead positions, so
//! parsers can count only the tokens relevant to their grammar.

use super::{TokenStream, lookahead::Lookahead};

/// A token stream that omits values selected by a predicate.
///
/// Read errors leave previously skipped tokens consumed, and lookahead counts
/// only retained tokens without guaranteeing isolation from shared state in
/// cloned sources or predicates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// NOTE(invariant): None.
pub struct Skipping<S, P> {
    /// Source whose tokens are tested and consumed.
    stream: S,

    /// Returns `true` for each token to consume without yielding.
    skip: P,
}

impl<S, P> Skipping<S, P> {
    /// Omit tokens from `stream` when `skip` returns `true`.
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
        let mut target_stream = self.clone();
        let mut target_remain = N;

        loop {
            let target_produced = TokenStream::next(&mut target_stream);

            match (target_remain, target_produced) {
                (_, Err(target_error)) => break Err(target_error),
                (_, Ok(None)) => break Ok(None),
                (0, Ok(Some(token))) => break Ok(Some(token)),
                (_, Ok(Some(_))) => target_remain = target_remain.saturating_sub(1),
            }
        }
    }
}

#[cfg(test)]
mod tests;
