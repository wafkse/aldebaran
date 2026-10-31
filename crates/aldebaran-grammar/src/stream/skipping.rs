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
    /// Underlying token stream.
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
mod tests {
    //! Tests module for the [`skipping`] module.
    //!
    //! [`skipping`]: super

    use crate::stream::{TokenStream, lookahead::Lookahead, slice::Slice};

    use super::Skipping;

    /// An interruption distinct from an empty token read.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Interrupted;

    /// A token sequence in which interruptions occupy positions alongside values.
    #[derive(Clone)]
    // NOTE(invariant): None.
    struct Fallible<'a>(Slice<'a, Result<u8, Interrupted>>);

    impl TokenStream for Fallible<'_> {
        type Token = u8;

        type Error = Interrupted;

        #[inline]
        fn next(&mut self) -> Result<Option<Self::Token>, Self::Error> {
            let Self(stream) = self;

            match stream.next() {
                Ok(token) => token.transpose(),
                Err(error) => match error {},
            }
        }
    }

    impl Lookahead for Fallible<'_> {
        #[inline]
        fn lookahead<const N: usize>(&self) -> Result<Option<Self::Token>, Self::Error> {
            let Self(stream) = self;

            match stream.lookahead::<N>() {
                Ok(token) => token.transpose(),
                Err(error) => match error {},
            }
        }
    }

    #[test]
    fn skips_selected_tokens_for_consumption_and_lookahead() {
        let tokens = [0_u8, 1, 0, 2];
        let stream = Slice::new(&tokens);
        let mut stream = Skipping::new(stream, |token: &u8| *token == 0);

        assert_eq!(stream.lookahead::<0>(), Ok(Some(1)));
        assert_eq!(stream.lookahead::<1>(), Ok(Some(2)));
        assert_eq!(stream.next(), Ok(Some(1)));
        assert_eq!(stream.next(), Ok(Some(2)));
        assert_eq!(stream.next(), Ok(None));
    }

    #[test]
    fn lookahead_reports_errors_without_consuming_them() {
        let tokens = [Ok(0), Ok(1), Ok(0), Err(Interrupted), Ok(2)];
        let stream = Fallible(Slice::new(&tokens));
        let mut stream = Skipping::new(stream, |token: &u8| *token == 0);

        assert_eq!(stream.lookahead::<1>(), Err(Interrupted));
        assert_eq!(stream.lookahead::<1>(), Err(Interrupted));
        assert_eq!(stream.next(), Ok(Some(1)));
        assert_eq!(stream.next(), Err(Interrupted));
        assert_eq!(stream.next(), Ok(Some(2)));
        assert_eq!(stream.next(), Ok(None));
    }

    #[test]
    fn entirely_filtered_input_exhausts_without_a_visible_token() {
        let stream = Slice::new(&[0_u8, 0, 0]);
        let mut stream = Skipping::new(stream, |token: &u8| *token == 0);

        assert_eq!(stream.lookahead::<0>(), Ok(None));
        assert_eq!(stream.lookahead::<{ usize::MAX }>(), Ok(None));
        assert_eq!(stream.next(), Ok(None));
        assert_eq!(stream.next(), Ok(None));
    }
}
