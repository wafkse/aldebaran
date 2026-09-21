//! Borrowed source views selected by absolute source spans.
//!
//! [`SourceDissect`] extends a source with a view type that remains iterable and
//! source-shaped. `dissect` selects one source interval, while `offset` derives a
//! view from an absolute boundary through the end of the source.

use core::num::NonZero;

use crate::{iter::SourceIter, source::Source};

use aldebaran_span::prelude::Span;

/// A trait for sources that can be dissected into smaller parts.
///
/// This is an *extension trait*, and thus only builds functionality on top of
/// [`Source`].
pub trait SourceDissect<'source>: Source<'source> {
    /// Narrow source view produced for one borrow.
    type View<'borrow>: Source<'borrow, Component = Self::Component> + SourceIter<'borrow>
    where
        Self: 'borrow;

    /// Retrieve the source window represented by `span`.
    fn dissect<'borrow>(&'borrow self, span: Span) -> Self::View<'borrow>;

    /// Retrieve a source view beginning at `offset` and extending to the end.
    #[inline]
    fn offset<'borrow>(&'borrow self, offset: usize) -> Option<Self::View<'borrow>> {
        let target_end = self.size();
        let target_len = target_end.saturating_sub(offset);

        match (offset, NonZero::new(target_len)) {
            (0, None) => None,
            (_, Some(length)) => Some(self.dissect(Span::new(offset, length))),
            (_, None) => None,
        }
    }
}
