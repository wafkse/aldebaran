//! Lexical recognition and construction over [`Cursor`].
//!
//! See the [`Lex`] trait for additional information.

use aldebaran_source::prelude::{Source, SourceDissect, SourceIter};

use super::Cursor;

/// A trait that expresses a two-step lexing operation.
///
/// This trait is for types that can be instantiated from a [`Cursor`], separating
/// lexical recognition from construction.
///
/// Implementations are expected to leave the cursor unchanged when returning an
/// error, so another lexical value can be tried at the same position. A
/// [`Cursor::branch`] can be used to keep reads speculative until they succeed.
///
/// # Two-stage Lexing
///
/// Lexing begins with [`pass`](Self::pass), which reads from the cursor and
/// collects the input needed to build [`Self`]. That input is then passed to
/// [`build_with_context`](Self::build_with_context), where the value is constructed
/// using a separately supplied context. This separation allows recognition to
/// finish before construction changes shared state, such as an interner.
///
/// For example, the pass for a string literal could use SIMD to scan runs of source
/// bytes, checking escape sequences as it encounters them. While locating the
/// closing quote and validating the literal, it can also count how many bytes the
/// decoded string will need. Returning that count alongside the source text lets
/// [`Lex::build_with_context`] allocate a buffer of exactly the required size
/// before decoding, without having to grow the buffer as it writes.
///
/// In this case, the scan does work that recognition already requires, while its
/// byte count tells construction how much space to allocate. The pass can therefore
/// prepare for an efficient allocation without allocating the buffer itself.
/// Because allocation is deferred until construction, recognition-only passes
/// avoid it. A rejected speculative parse can likewise discard the recognized
/// input without acquiring resources needed only to build the value.
///
/// The pass should only collect the information needed to recognize or build the
/// value. Consequently, semantic conversion, allocation, interning, and final
/// structure construction belong in [`Lex::build_with_context`] whenever they can
/// be deferred without changing recognition, while useful results of the scan can
/// be carried in [`Input`](Self::Input) for construction to reuse.
///
/// # Borrowing
///
/// The input returned by [`pass`](Self::pass) may borrow for the lifetime of that
/// call's cursor borrow. If it does, the cursor can remain borrowed until the
/// input is no longer used, even though recognition has already finished.
pub trait Lex<'source, S>
where
    S: Source<'source> + SourceDissect<'source> + SourceIter<'source> + ?Sized,
{
    /// The input needed to build [`Self`] after reading it from the source.
    ///
    /// This input may borrow source data when construction needs it, but the
    /// construction context is supplied separately to [`Lex::build_with_context`].
    ///
    /// Because construction belongs to that second stage, the input should not
    /// contain a prebuilt [`Self`] merely to make construction trivial.
    type Input<'input>
    where
        'source: 'input;

    /// The context used while lexical values are constructed, such as an interner.
    type Context: ?Sized;

    /// The type of the error associated with this lexing operation.
    type Error;

    /// Recognize source text and produce only the input required by
    /// [`Lex::build_with_context`].
    ///
    /// This method reads through the [`Cursor`], leaving construction work to
    /// [`Lex::build_with_context`] whenever it can be deferred without changing
    /// recognition.
    ///
    /// # Errors
    ///
    /// Returns an error if the source text cannot be recognized as this lexical
    /// value. Implementations are expected to leave the cursor at its original
    /// position when this happens, so another value can be tried there.
    fn pass<'input>(text: &'input mut Cursor<'source, S>) -> Result<Self::Input<'input>, Self::Error>
    where
        'source: 'input;

    /// Perform recognition and construction with one persistent context.
    ///
    /// # Errors
    ///
    /// If [`Self::pass`] fails, its error is returned without attempting
    /// construction. Otherwise, the result comes from [`Self::build_with_context`].
    ///
    /// Implementations are expected to leave the cursor unchanged on error.
    /// The default implementation relies on [`Self::pass`] to do this when
    /// recognition fails, but does not restore the cursor if construction fails.
    /// Implementations whose construction can fail after advancing the cursor
    /// therefore need to override this method to meet that expectation. Changes
    /// to the context must be handled separately.
    #[inline]
    fn lex_with_context(text: &mut Cursor<'source, S>, context: &mut Self::Context) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let input = Self::pass(text)?;

        Self::build_with_context(input, context)
    }

    /// Perform recognition and construction with a default context.
    ///
    /// Each call creates its own default context, so changes to that context are
    /// not carried over to later calls.
    ///
    /// # Errors
    ///
    /// Returns the error from [`Self::lex_with_context`], which is expected to
    /// leave the cursor unchanged on failure.
    #[inline]
    fn lex(text: &mut Cursor<'source, S>) -> Result<Self, Self::Error>
    where
        Self::Context: Default,
        Self: Sized,
    {
        let mut context = Default::default();

        Self::lex_with_context(text, &mut context)
    }

    /// Build [`Self`] from the corresponding [`Lex::Input`] with context.
    ///
    /// Semantic conversion, allocation, interning, and final structure
    /// construction belong here when the pass can defer them.
    ///
    /// The input normally comes from [`Self::pass`]. If you construct it directly
    /// instead, it must satisfy the same assumptions that this implementation
    /// makes during recognition.
    ///
    /// # Errors
    ///
    /// Returns an error if the recognized input cannot be used to build [`Self`],
    /// even if recognition succeeded. Any changes already made to the context are
    /// not automatically undone when this happens.
    fn build_with_context<'input>(input: Self::Input<'input>, context: &mut Self::Context) -> Result<Self, Self::Error>
    where
        'source: 'input,
        Self: Sized;

    /// Build [`Self`] with a default context.
    ///
    /// Each call creates its own default context, so changes to that context are
    /// not carried over to later calls.
    ///
    /// # Errors
    ///
    /// Returns the error from [`Self::build_with_context`].
    #[inline]
    fn build<'input>(input: Self::Input<'input>) -> Result<Self, Self::Error>
    where
        Self::Context: Default,
        'source: 'input,
        Self: Sized,
    {
        let mut context = Default::default();

        Self::build_with_context(input, &mut context)
    }
}

#[cfg(test)]
mod tests {
    //! Tests module for the [`consume`] module.
    //!
    //! [`consume`]: super

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
}
