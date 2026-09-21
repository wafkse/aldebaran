//! Relevant and irrelevant pieces produced while segmenting one source line.
//!
//! [`Piecewise`] preserves a source view for every interval in the line and
//! attaches caller data only to intervals that intersect that data's span. This
//! lets renderers walk the entire line without separately reconstructing gaps.

use core::{fmt, hash};

use aldebaran_span::prelude::{Span, Spanned};

use crate::line::{LineSegmented, Segment, SourceLines, segment::IrrelevantSegment};

/// One interval in a piecewise decomposition of a source line.
///
/// Relevant intervals retain the spanned data responsible for the split.
/// Irrelevant intervals represent the source gaps between those values, so the
/// complete sequence can reconstruct the original line without missing regions.
pub enum Piecewise<'source, S, D>
where
    S: SourceLines<'source> + ?Sized,
    D: Spanned,
{
    /// A relevant line segment.
    Relevant(Segment<'source, S, D>),

    /// An irrelevant line segment.
    Irrelevant(IrrelevantSegment<'source, S>),
}

impl<'source, S, D> Spanned for Piecewise<'source, S, D>
where
    S: SourceLines<'source> + ?Sized,
    D: Spanned,
{
    #[inline]
    fn span(&self) -> Span {
        match self {
            Piecewise::Relevant(relevant_segment) => relevant_segment.span(),
            Piecewise::Irrelevant(irrelevant_segment) => irrelevant_segment.span(),
        }
    }
}

impl<'source, S, D> Piecewise<'source, S, D>
where
    S: SourceLines<'source> + ?Sized,
    D: Spanned,
{
    /// Retrieve the view into the source line regardless of its relevance.
    #[inline]
    pub const fn view(&self) -> &<S::Line as LineSegmented<'source, S>>::View {
        match self {
            Self::Relevant(segment) => segment.view(),
            Self::Irrelevant(segment) => segment.view(),
        }
    }

    /// Retrieve a reference to the internally-stored data if it is relevant.
    #[inline]
    pub const fn data(&self) -> Option<&D> {
        match self {
            Self::Relevant(segment) => Some(segment.data()),
            Self::Irrelevant(_) => None,
        }
    }
}

impl<'source, S, D> fmt::Debug for Piecewise<'source, S, D>
where
    S: SourceLines<'source> + ?Sized,
    D: Spanned,
    <S::Line as LineSegmented<'source, S>>::View: fmt::Debug,
    D: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Relevant(segment) => f.debug_tuple("Relevant").field(segment).finish(),
            Self::Irrelevant(segment) => f.debug_tuple("Irrelevant").field(segment).finish(),
        }
    }
}

impl<'source, S, D> Clone for Piecewise<'source, S, D>
where
    S: SourceLines<'source> + ?Sized,
    D: Spanned,
    <S::Line as LineSegmented<'source, S>>::View: Clone,
    D: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        match self {
            Self::Relevant(segment) => Self::Relevant(segment.clone()),
            Self::Irrelevant(segment) => Self::Irrelevant(segment.clone()),
        }
    }
}

impl<'source, S, D> Copy for Piecewise<'source, S, D>
where
    S: SourceLines<'source> + ?Sized,
    D: Spanned,
    <S::Line as LineSegmented<'source, S>>::View: Copy,
    D: Copy,
{
}

impl<'source, S, D> PartialEq for Piecewise<'source, S, D>
where
    S: SourceLines<'source> + ?Sized,
    D: Spanned,
    <S::Line as LineSegmented<'source, S>>::View: PartialEq,
    D: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Relevant(segment), Self::Relevant(other_segment)) => segment == other_segment,
            (Self::Irrelevant(segment), Self::Irrelevant(other_segment)) => segment == other_segment,
            _ => false,
        }
    }
}

impl<'source, S, D> Eq for Piecewise<'source, S, D>
where
    S: SourceLines<'source> + ?Sized,
    D: Spanned,
    <S::Line as LineSegmented<'source, S>>::View: Eq,
    D: Eq,
{
}

impl<'source, S, D> hash::Hash for Piecewise<'source, S, D>
where
    S: SourceLines<'source> + ?Sized,
    D: Spanned,
    <S::Line as LineSegmented<'source, S>>::View: hash::Hash,
    D: hash::Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Relevant(segment) => segment.hash(state),
            Self::Irrelevant(segment) => segment.hash(state),
        }
    }
}
