//! Pairing of a source cursor with borrowed mutable construction state.
//!
//! [`Internment`] keeps one [`Text`] cursor and one mutable context borrow alive
//! together. Lexical streams use this carrier to reuse storage or other state
//! across successive values without embedding that state in individual tokens.

use aldebaran_source::prelude::{SourceDissect, SourceIter};

use super::Text;

/// A text cursor paired with one borrowed mutable interner.
#[derive(Debug)]
pub struct Internment<'source, 'interner, S, I>
where
    S: SourceDissect<'source> + SourceIter<'source> + ?Sized,
    I: ?Sized,
{
    /// Source cursor owned by this carrier.
    text: Text<'source, S>,

    /// Interner borrowed while the source cursor is consumed.
    interner: &'interner mut I,
}

// NOTE(invariant): `text` is the only cursor for the paired source and `interner`
// remains mutably borrowed for exactly the carrier lifetime. Consuming or dropping
// the carrier releases the interner borrow without transferring it into outputs.

impl<'source, 'interner, S, I> Internment<'source, 'interner, S, I>
where
    S: SourceDissect<'source> + SourceIter<'source> + ?Sized,
    I: ?Sized,
{
    /// Create a source cursor paired with borrowed mutable internment state.
    #[inline]
    #[must_use]
    pub const fn new(source: &'source S, interner: &'interner mut I) -> Self {
        let text = Text::create(source);

        Self { text, interner }
    }

    /// Borrow the source cursor and interner together.
    #[inline]
    pub const fn parts_mut(&mut self) -> (&mut Text<'source, S>, &mut I) {
        let Self { text, interner } = self;

        let interner = &mut **interner;

        (text, interner)
    }
}
