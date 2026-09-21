//! Lexical recognition and construction over [`Text`](super::Text).
//!
//! [`Lex`] defines the source-consumption protocol used by lexical values. The
//! pass stage carries only source-derived information required by construction,
//! while persistent mutable context is supplied separately during value building.

use aldebaran_source::prelude::{Source, SourceDissect, SourceIter};

use super::Text;

/// A trait that expresses a two-step lexing operation.
///
/// The pass produces only the source-derived input required to build [`Self`].
/// Semantic conversion, allocation, interning, and final structure construction
/// belong in [`Lex::build_with_ctx`]. Persistent context is supplied separately
/// during construction.
///
/// This trait is for types that can be instantiated from a [`Text`], separating
/// lexical recognition from construction.
///
/// Recognition may still require a complete source scan. For example, an inline
/// string pass must locate its closing quote and validate escape syntax. The
/// decoded byte buffer is not required to recognize that source extent, so its
/// construction belongs in [`Lex::build_with_ctx`].
///
/// Pass-time work is justified only when the resulting fact must be transported
/// to build the final value. Optimization hints that can be recomputed during
/// construction do not belong in the pass.
pub trait Lex<'source, S>
where
    S: Source<'source> + SourceDissect<'source> + SourceIter<'source> + ?Sized,
{
    /// The least source-derived input required to build [`Self`].
    ///
    /// This may borrow source data when construction needs it. Persistent
    /// construction context is supplied separately to [`Lex::build_with_ctx`].
    /// It should not contain a prebuilt [`Self`] merely to make construction
    /// trivial.
    type Input<'input>
    where
        'source: 'input;

    /// Persistent mutable context used while lexical values are constructed.
    type Context: ?Sized;

    /// The type of the error associated with this lexing operation.
    type Error;

    /// Recognize source text and produce only the input required by
    /// [`Lex::build_with_ctx`].
    ///
    /// Construction work that can be deferred without changing recognition
    /// belongs in [`Lex::build_with_ctx`].
    ///
    /// See [`Text`] for more information.
    fn pass<'input>(text: &'input mut Text<'source, S>) -> Result<Self::Input<'input>, Self::Error>
    where
        'source: 'input;

    /// Perform recognition and construction with one persistent context.
    #[inline]
    fn lex_with_ctx(text: &mut Text<'source, S>, context: &mut Self::Context) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let input = Self::pass(text)?;

        Self::build_with_ctx(input, context)
    }

    /// Perform recognition and construction with a default context.
    #[inline]
    fn lex(text: &mut Text<'source, S>) -> Result<Self, Self::Error>
    where
        Self::Context: Default,
        Self: Sized,
    {
        let mut context = Default::default();

        Self::lex_with_ctx(text, &mut context)
    }

    /// Build [`Self`] from the corresponding [`Lex::Input`] with context.
    ///
    /// Semantic conversion, allocation, interning, and final structure
    /// construction belong here when the pass can defer them. This operation may
    /// fail.
    fn build_with_ctx<'input>(input: Self::Input<'input>, context: &mut Self::Context) -> Result<Self, Self::Error>
    where
        'source: 'input,
        Self: Sized;

    /// Build [`Self`] with a default context.
    #[inline]
    fn build<'input>(input: Self::Input<'input>) -> Result<Self, Self::Error>
    where
        Self::Context: Default,
        'source: 'input,
        Self: Sized,
    {
        let mut context = Default::default();

        Self::build_with_ctx(input, &mut context)
    }
}
