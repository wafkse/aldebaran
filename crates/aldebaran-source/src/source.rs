//! Base source abstraction and source-plane coordinates.
//!
//! Iteration, dissection, metadata, and line access are layered on [`Source`]
//! through independent capabilities.

use core::num::NonZero;

use aldebaran_baseline::prelude::Span;

use crate::{component::Component, terminate::Terminated};

/// The default line-terminator type associated with a source.
pub type DefaultTerminator<'a, S> = <S as Terminated<'a, S>>::TerminatedBy;

/// Coordinates shared by a source sequence and its component traversal.
///
/// Component widths and spans use the same unit so traversal can remain aligned
/// with source locations. Built-in strings therefore report UTF-8 byte coordinates
/// even though iteration yields [`char`] values, whose widths may differ.
pub trait Source<'a>
where
    Self: 'a,
{
    /// Traversal unit whose width determines distances in this source's coordinate plane.
    type Component: Component;

    /// The source span, or `None` when the source is empty.
    ///
    /// A nonempty footprint begins at coordinate zero, and its length equals
    /// the sum of the source components' widths from [`Component::size`].
    fn footprint(&self) -> Option<Span>;

    /// Source extent in coordinate units, counting UTF-8 bytes for strings.
    /// Empty sources have size zero even though their start and end anchors are unit spans.
    #[inline]
    fn size(&self) -> usize {
        self.footprint().as_ref().map(Span::length).map(NonZero::get).unwrap_or(usize::MIN)
    }

    /// Unit anchor at the first source coordinate.
    /// For empty input the anchor is at zero and does not identify source content.
    #[inline]
    fn start(&self) -> Span {
        let span_start = self.footprint().as_ref().map(Span::start).unwrap_or(usize::MIN);

        Span::unit(span_start)
    }

    /// Unit anchor at the last occupied coordinate, before the exclusive source end.
    ///
    /// This can lie inside a multi-byte component rather than at its start.
    /// Empty input uses zero as an anchor without identifying source content.
    #[inline]
    fn end(&self) -> Span {
        let span_end = self.footprint().as_ref().map(Span::end).unwrap_or(0);

        Span::unit(span_end.saturating_sub(1))
    }
}

/// Blanket implementation for references to sources.
impl<'source, 'borrow, S> Source<'source> for &'borrow S
where
    'source: 'borrow,
    S: Source<'source> + ?Sized,
    &'borrow S: 'source,
{
    type Component = S::Component;

    #[inline]
    fn footprint(&self) -> Option<Span> {
        S::footprint(self)
    }
}
