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
