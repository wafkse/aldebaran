//! Annotated line windows and gaps between them.
//!
//! [`Piecewise::Relevant`] retains caller data through clipping, while
//! [`Piecewise::Irrelevant`] carries only a gap's absolute span. Renderers can
//! use [`Piecewise::view`] for either and inspect data only for annotations.

use core::{fmt, hash};

use aldebaran_baseline::prelude::{Span, Spanned};

use crate::line::{LineSegmented, Segment, SourceLines, segment::IrrelevantSegment};

/// Renderable line content distinguished by annotation relevance.
///
/// Relevant pieces retain the annotation's original span even if the view is
/// clipped. Gaps carry their own absolute source span without annotation data.
pub enum Piecewise<'source, S, D>
where
    S: SourceLines<'source> + ?Sized,
    D: Spanned,
{
    /// Content selected by caller data whose original span is retained.
    Relevant(Segment<'source, S, D>),

    /// Unannotated content carrying its own absolute source extent.
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
    /// Renderable content shared by annotations and gaps.
    /// An annotation's view may be clipped without changing its reported source span.
    #[inline]
    pub const fn view(&self) -> &<S::Line as LineSegmented<'source, S>>::View {
        match self {
            Self::Relevant(segment) => segment.view(),
            Self::Irrelevant(segment) => segment.view(),
        }
    }

    /// Return the associated data for a relevant interval, or `None` for a gap.
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
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
        match self {
            Self::Relevant(segment) => segment.hash(state),
            Self::Irrelevant(segment) => segment.hash(state),
        }
    }
}
