//! Pairing of a source cursor with borrowed mutable construction state.
//!
//! See [`Internment`] for further information.

use core::fmt;

use aldebaran_source::prelude::{SourceDissect, SourceIter};

use super::Cursor;

/// A text cursor paired with one borrowed mutable construction context.
///
/// Lexical streams use this carrier to keep the source position and construction
/// state together between reads.
// NOTE(invariant): None.
pub struct Internment<'source, 'interner, S, I>
where
    S: SourceDissect<'source> + SourceIter<'source> + ?Sized,
    I: ?Sized,
{
    /// Source cursor owned by this carrier.
    source_text: Cursor<'source, S>,

    /// Construction context borrowed while the source cursor is consumed.
    internment_structure: &'interner mut I,
}

impl<'source, 'interner, S, I> Internment<'source, 'interner, S, I>
where
    S: SourceDissect<'source> + SourceIter<'source> + ?Sized,
    I: ?Sized,
{
    /// Create a source cursor paired with borrowed mutable construction state.
    #[inline]
    #[must_use]
    pub const fn new(source: &'source S, internment_structure: &'interner mut I) -> Self {
        let source_text = Cursor::create(source);

        Self {
            source_text,
            internment_structure,
        }
    }

    /// Borrow the source cursor and construction context together.
    ///
    /// Changes to either state remain available to subsequent reads.
    #[inline]
    pub const fn parts_mut(&mut self) -> (&mut Cursor<'source, S>, &mut I) {
        let Self {
            source_text: text,
            internment_structure: interner,
        } = self;

        let interner = &mut **interner;

        (text, interner)
    }
}

impl<'source, 'interner, S, I> fmt::Debug for Internment<'source, 'interner, S, I>
where
    S: SourceDissect<'source> + SourceIter<'source> + ?Sized,
    S::Component: fmt::Debug,
    I: fmt::Debug + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            source_text,
            internment_structure,
        } = self;

        f.debug_struct("Internment")
            .field("source_text", source_text)
            .field("internment_structure", internment_structure)
            .finish()
    }
}
