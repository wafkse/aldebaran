//! Coordinate-based source views for traversal and rendering.
//!
//! [`SourceDissect`] selects a region in the original source plane and exposes
//! it as an iterable source with the same component type. Built-in slices
//! measure the resulting view from zero while callers retain the original
//! span separately.

use core::num::NonZero;

use crate::{iter::SourceIter, source::Source};

use aldebaran_baseline::prelude::Span;

/// Source windows selected without changing the component representation.
/// Implementations can borrow the selected content for the dissection call's lifetime.
/// The built-in string and byte views borrow directly from that source.
pub trait SourceDissect<'source>: Source<'source> {
    /// Iterable view preserving component identity for the selected region.
    type View<'borrow>: Source<'borrow, Component = Self::Component> + SourceIter<'borrow>
    where
        Self: 'borrow;

    /// Return a view of the source window selected by an absolute source span.
    ///
    /// Because selection uses this source's coordinate plane, callers must supply
    /// coordinates belonging to that plane. Out-of-range handling is left to the
    /// implementation, and [`str`] imposes a further boundary requirement because
    /// slicing panics unless both endpoints lie on UTF-8 boundaries.
    fn dissect<'borrow>(&'borrow self, span: Span) -> Self::View<'borrow>;

    /// Return a view beginning at `offset` and extending to the source end.
    ///
    /// Empty sources and offsets at or beyond the source end yield `None`.
    /// Built-in strings require a UTF-8 boundary and panic on an interior byte offset.
    #[inline]
    fn offset<'borrow>(&'borrow self, offset: usize) -> Option<Self::View<'borrow>> {
        NonZero::new(self.size().saturating_sub(offset)).map(|length| self.dissect(Span::new(offset, length)))
    }
}
