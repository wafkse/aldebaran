use alloc::vec::Vec;
use core::num::NonZero;

use aldebaran_baseline::span::Span;

use crate::{builtin::property::AsciiDigit, cursor::error::Expected};

use super::{Cursor, Lex};

/// A nonzero decimal digit whose stored numeric value is independent of its ASCII spelling.
#[derive(Debug, PartialEq, Eq)]
// NOTE(invariant): None.
struct NonzeroDigit(NonZero<u8>);

/// Distinguishes unavailable digit syntax from a zero value rejected during construction.
#[derive(Debug, PartialEq, Eq)]
enum Failure {
    /// An ASCII digit was required at the reported source position.
    Expected(Expected<u8, AsciiDigit>),

    /// The recognized digit was zero.
    Zero,
}

impl<'source> Lex<'source, [u8]> for NonzeroDigit {
    type Input<'input>
        = u8
    where
        'source: 'input;

    type Context = Vec<u8>;

    type Error = Failure;

    #[inline]
    fn pass<'input>(text: &'input mut Cursor<'source, [u8]>) -> Result<Self::Input<'input>, Self::Error>
    where
        'source: 'input,
    {
        text.expect_is_next(AsciiDigit).map(|byte| byte - b'0').map_err(Failure::Expected)
    }

    #[inline]
    fn build_with_context<'input>(input: Self::Input<'input>, attempts: &mut Self::Context) -> Result<Self, Self::Error>
    where
        'source: 'input,
    {
        attempts.push(input);

        NonZero::new(input).map(Self).ok_or(Failure::Zero)
    }
}

#[test]
fn recognition_failure_skips_construction_and_keeps_consumption() {
    let mut text = Cursor::create(b"x1".as_slice());
    let mut attempts = Vec::new();
    let expected = Expected::found(AsciiDigit, b'x', Span::unit(0));

    assert_eq!(
        NonzeroDigit::lex_with_context(&mut text, &mut attempts),
        Err(Failure::Expected(expected))
    );
    assert_eq!(text.index(), 1);
    assert!(attempts.is_empty(), "recognition failure must not invoke construction");

    let digit = NonzeroDigit::lex_with_context(&mut text, &mut attempts).expect("the next byte is a nonzero digit");

    assert_eq!(digit, NonzeroDigit(NonZero::<u8>::MIN));
    assert_eq!(attempts, [1]);
    assert_eq!(text.index(), 2);
}

#[test]
fn construction_failure_preserves_consumption_and_context_effects() {
    let mut text = Cursor::create(b"01".as_slice());
    let mut attempts = Vec::new();

    assert_eq!(NonzeroDigit::lex_with_context(&mut text, &mut attempts), Err(Failure::Zero));
    assert_eq!(text.index(), 1);
    assert_eq!(text.peek(), Some(b'1'));
    assert_eq!(attempts, [0]);

    let digit = NonzeroDigit::lex_with_context(&mut text, &mut attempts).expect("the next digit can be constructed");

    assert_eq!(digit, NonzeroDigit(NonZero::<u8>::MIN));
    assert_eq!(attempts, [0, 1]);
    assert_eq!(text.index(), 2);
}

#[test]
fn recognition_can_finish_before_construction_borrows_context() {
    let mut text = Cursor::create(b"2!".as_slice());
    let input = NonzeroDigit::pass(&mut text).expect("the source starts with a digit");

    assert_eq!(text.index(), 1);
    assert_eq!(text.peek(), Some(b'!'));

    let mut attempts = Vec::new();
    let digit = NonzeroDigit::build_with_context(input, &mut attempts).expect("two is nonzero");

    let NonzeroDigit(value) = digit;

    assert_eq!(value.get(), 2);
    assert_eq!(attempts, [2]);
    assert_eq!(text.index(), 1);
}
