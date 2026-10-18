//! Recognition and construction of lexical values over [`Cursor`]. [`Lex`] permits
//! recognition without construction state, so callers can postpone semantic work
//! until they accept the recognized syntax.

use aldebaran_source::prelude::{Source, SourceDissect, SourceIter};

use super::Cursor;

/// Defines how a lexical value is recognized from source and constructed from its
/// recognized input. Implement this trait on the value produced by lexing.
///
/// Lexing has two stages. [`pass`](Self::pass) recognizes the value's syntax through
/// a [`Cursor`] and returns [`Input`](Self::Input), retaining the spelling or parsed
/// components needed for construction. It has no access to [`Context`](Self::Context).
/// [`build_with_context`](Self::build_with_context) turns that input into `Self`
/// using caller-owned state, such as an interner. Construction receives no separate
/// cursor argument, so its source data must be retained in the intermediate input.
///
/// Separating these stages lets a parser recognize syntax before deciding whether
/// to construct its semantic value and modify shared state. For example, identifier
/// recognition can retain its source spelling while deferring interning until the
/// parser accepts that identifier. Input may borrow for the recognition call's
/// lifetime, so retaining it can also retain the cursor borrow. Callers that want
/// immediate construction can use [`lex_with_context`](Self::lex_with_context).
///
/// Neither stage provides automatic rollback. Implementations define cursor
/// progress when recognition fails, and construction can fail after recognition
/// succeeds or after modifying context. Recognizing valid syntax therefore does
/// not guarantee that its value can be constructed. Use [`Cursor::branch`] when recognition
/// needs speculative cursor progress, and manage context effects separately.
pub trait Lex<'source, S>
where
    S: Source<'source> + SourceDissect<'source> + SourceIter<'source> + ?Sized,
{
    /// Data retained from recognition for later construction, possibly borrowed for
    /// the recognition call's lifetime.
    type Input<'input>
    where
        'source: 'input;

    /// Caller-owned state shared across constructions, such as an interner.
    type Context: ?Sized;

    /// Failure shared by syntax recognition and semantic construction.
    type Error;

    /// Establish the input needed for construction without access to construction
    /// context. Each implementation defines its cursor advancement on failure.
    fn pass<'input>(text: &'input mut Cursor<'source, S>) -> Result<Self::Input<'input>, Self::Error>
    where
        'source: 'input;

    /// Recognize and construct a value with caller-owned state. The default
    /// implementation retains cursor progress and context effects on error.
    #[inline]
    fn lex_with_context(text: &mut Cursor<'source, S>, context: &mut Self::Context) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let input = Self::pass(text)?;

        Self::build_with_context(input, context)
    }

    /// Recognize and construct using a default context local to this call,
    /// rather than retaining the context for later lexical reads.
    #[inline]
    fn lex(text: &mut Cursor<'source, S>) -> Result<Self, Self::Error>
    where
        Self::Context: Default,
        Self: Sized,
    {
        let mut context = Default::default();

        Self::lex_with_context(text, &mut context)
    }

    /// Interpret previously recognized input using caller-owned state, with no
    /// automatic rollback of state changes on failure.
    ///
    /// Supply input returned by [`Self::pass`], or uphold the same implementation's
    /// recognition assumptions when assembling input directly.
    fn build_with_context<'input>(input: Self::Input<'input>, context: &mut Self::Context) -> Result<Self, Self::Error>
    where
        'source: 'input,
        Self: Sized;

    /// Interpret previously recognized input using a default context local to this call.
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
mod tests;
