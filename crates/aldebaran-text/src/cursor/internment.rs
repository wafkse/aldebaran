//! Shared state for lexical reads. [`Internment`] keeps source position and
//! caller-owned construction context available together across token production.

use aldebaran_source::prelude::{SourceDissect, SourceIter};

use super::Cursor;

/// Maintains source progress alongside a borrowed construction context across
/// lexical reads. [`Self::parts_mut`] permits coordinated access to both states.
#[derive(Debug)]
// NOTE(invariant): None.
pub struct Internment<'source, 'interner, S, I>
where
    S: SourceDissect<'source> + SourceIter<'source> + ?Sized,
    I: ?Sized,
{
    /// Source position retained between lexical reads.
    text: Cursor<'source, S>,

    /// Caller-owned context whose construction effects persist across reads.
    interner: &'interner mut I,
}

impl<'source, 'interner, S, I> Internment<'source, 'interner, S, I>
where
    S: SourceDissect<'source> + SourceIter<'source> + ?Sized,
    I: ?Sized,
{
    /// Begin traversal at source offset zero while retaining the caller's construction context.
    #[inline]
    #[must_use]
    pub const fn new(source: &'source S, interner: &'interner mut I) -> Self {
        let text = Cursor::create(source);

        Self { text, interner }
    }

    /// Access source progress and construction context together for a lexical read.
    /// Changes to either remain available to subsequent reads.
    #[inline]
    pub const fn parts_mut(&mut self) -> (&mut Cursor<'source, S>, &mut I) {
        let Self { text, interner } = self;

        let interner = &mut **interner;

        (text, interner)
    }
}
