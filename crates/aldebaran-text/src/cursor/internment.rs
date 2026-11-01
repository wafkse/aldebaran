//! Pairing of a source cursor with borrowed mutable construction state.
//!
//! See [`Internment`] for further information.

use aldebaran_source::prelude::{SourceDissect, SourceIter};

use super::Cursor;

/// A text cursor paired with one borrowed mutable construction context.
///
/// Lexical streams use this carrier to keep the source position and construction
/// state together between reads.
#[derive(Debug)]
// NOTE(invariant): None.
pub struct Internment<'source, 'interner, S, I>
where
    S: SourceDissect<'source> + SourceIter<'source> + ?Sized,
    I: ?Sized,
{
    /// Source cursor owned by this carrier.
    text: Cursor<'source, S>,

    /// Construction context borrowed while the source cursor is consumed.
    interner: &'interner mut I,
}

impl<'source, 'interner, S, I> Internment<'source, 'interner, S, I>
where
    S: SourceDissect<'source> + SourceIter<'source> + ?Sized,
    I: ?Sized,
{
    /// Create a source cursor paired with borrowed mutable construction state.
    #[inline]
    #[must_use]
    pub const fn new(source: &'source S, interner: &'interner mut I) -> Self {
        let text = Cursor::create(source);

        Self { text, interner }
    }

    /// Borrow the source cursor and construction context together.
    ///
    /// Changes to either state remain available to subsequent reads.
    #[inline]
    pub const fn parts_mut(&mut self) -> (&mut Cursor<'source, S>, &mut I) {
        let Self { text, interner } = self;

        let interner = &mut **interner;

        (text, interner)
    }
}
