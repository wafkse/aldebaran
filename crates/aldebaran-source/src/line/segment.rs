//! Projection of spanned data onto individual source lines.
//!
//! See [`LineSegmented`] and [`Segmented`] for further information.

pub mod iter;
pub mod piecewise;

pub use piecewise::Piecewise;

pub use iter::Segmented;

use core::{cmp, fmt, hash, iter::FusedIterator};

use aldebaran_baseline::prelude::{Span, Spanned};

use crate::line::{LineId, SourceLines};

/// Annotation projection for source-specific line representations.
/// Renderers use its windows for content and retain spanned data for presentation decisions.
pub trait LineSegmented<'source, S>
where
    S: SourceLines<'source, Line = Self> + ?Sized,
{
    /// The iterator type produced for segmented input data.
    ///
    /// No order is implied for the segments, so users must sort them if they
    /// require a particular order.
    type Segments<'input, I, D>: Iterator<Item = Piecewise<'source, S, D>>
    where
        I: Iterator<Item = D> + FusedIterator,
        S: 'input,
        D: Spanned,
        'source: 'input;

    /// A view into a source line.
    ///
    /// This is provided as a convenience for higher-kinded abstraction. Thus,
    /// in most cases, it will be the same as the source line. In a generic
    /// context it serves as a token type for a view into the source line, but
    /// not the source line itself.
    type View;

    /// Segment the source line with the given spanned data.
    ///
    /// No order is imposed on the input [`spans`](Span), so the segments can be
    /// in any order. Implementations define how overlapping or duplicate spans
    /// are represented, and may ignore spans that do not intersect the source.
    fn segmented<'input, I, D>(line: &'input LineId<'source, S>, data_iter: I) -> Self::Segments<'input, I, D>
    where
        'source: 'input,
        I: Iterator<Item = D> + FusedIterator,
        D: Spanned;

    /// Retrieve a view into the source line for the given span.
    ///
    /// `None` means the window is out of bounds or not representable. For
    /// string views both endpoints must be UTF-8 boundaries.
    fn window(&self, span: Span) -> Option<Self::View>;

    /// Borrow the complete content when its source coordinates are UTF-8 byte offsets.
    ///
    /// The returned text must match this line's visualization exactly, and every
    /// byte offset must retain its original source coordinate. Other line
    /// representations leave this unavailable and use their windows instead.
    #[inline]
    fn as_str(&self) -> Option<&str> {
        None
    }
}

/// A single relevant line segment.
///
/// This encapsulates a view into the source line with some data `D`.
///
/// The view may be clipped to line content while the data keeps its original
/// source span. Consequently [`Spanned::span`] reports the annotation's extent,
/// which can be larger than the rendered window.
// NOTE(invariant): OriginalSourceRange(LineView) SubsetOf Data.Span
pub struct Segment<'a, S, D>
where
    S: SourceLines<'a> + ?Sized,
    D: Spanned,
{
    /// The view into the source line.
    line_view: <S::Line as LineSegmented<'a, S>>::View,

    /// The underlying [`Spanned`] data of the segment.
    data: D,
}

impl<'a, S, D> Segment<'a, S, D>
where
    S: SourceLines<'a> + ?Sized,
    D: Spanned,
{
    /// Retrieve the view into the source line.
    #[inline]
    pub const fn view(&self) -> &<S::Line as LineSegmented<'a, S>>::View {
        let &Self { ref line_view, .. } = self;

        line_view
    }

    /// Retrieve a reference to the internally-stored data.
    ///
    /// The data keeps its original source span even when the line view is clipped.
    #[inline]
    pub const fn data(&self) -> &D {
        let &Self { ref data, .. } = self;

        data
    }

    /// Construct a new [`Segment`] from its raw parts: the line view and the
    /// data.
    ///
    /// The caller must keep the view within the data's span in the original source.
    #[inline]
    pub const fn raw(line_view: <S::Line as LineSegmented<'a, S>>::View, data: D) -> Self {
        Self { line_view, data }
    }

    /// Deconstruct this segment into its raw parts: the line view and the data.
    #[inline]
    pub fn partitioned(self) -> (<S::Line as LineSegmented<'a, S>>::View, D) {
        let Self { line_view, data } = self;

        (line_view, data)
    }
}

impl<'a, S, D> Spanned for Segment<'a, S, D>
where
    S: SourceLines<'a> + ?Sized,
    D: Spanned,
{
    #[inline]
    fn span(&self) -> Span {
        let &Self { ref data, .. } = self;

        data.span()
    }
}

impl<'a, S, D> hash::Hash for Segment<'a, S, D>
where
    S: SourceLines<'a> + ?Sized,
    D: Spanned,
    <S::Line as LineSegmented<'a, S>>::View: hash::Hash,
    D: hash::Hash,
{
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
        let &Self { ref line_view, ref data } = self;

        line_view.hash(state);
        data.hash(state);
    }
}

impl<'a, S, D> Ord for Segment<'a, S, D>
where
    S: SourceLines<'a> + ?Sized,
    D: Spanned,
    <S::Line as LineSegmented<'a, S>>::View: Ord,
    D: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let &Self { ref line_view, ref data } = self;

        let &Self {
            line_view: ref other_line,
            data: ref other_data,
        } = other;

        match line_view.cmp(other_line) {
            cmp::Ordering::Equal => data.cmp(other_data),
            target_value => target_value,
        }
    }
}

impl<'a, S, D> PartialOrd for Segment<'a, S, D>
where
    S: SourceLines<'a> + ?Sized,
    D: Spanned,
    <S::Line as LineSegmented<'a, S>>::View: PartialOrd,
    D: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let &Self { ref line_view, ref data } = self;

        let &Self {
            line_view: ref other_line,
            data: ref other_data,
        } = other;

        match line_view.partial_cmp(other_line) {
            Some(cmp::Ordering::Equal) => data.partial_cmp(other_data),
            target_value => target_value,
        }
    }
}

impl<'a, S, D> Eq for Segment<'a, S, D>
where
    S: SourceLines<'a> + ?Sized,
    D: Spanned,
    <S::Line as LineSegmented<'a, S>>::View: Eq,
    D: Eq,
{
}

impl<'a, S, D> PartialEq for Segment<'a, S, D>
where
    S: SourceLines<'a> + ?Sized,
    D: Spanned,
    <S::Line as LineSegmented<'a, S>>::View: PartialEq,
    D: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let &Self {
            line_view: ref line,
            ref data,
        } = self;

        let &Self {
            line_view: ref other_line,
            data: ref other_data,
        } = other;

        line == other_line && data == other_data
    }
}

impl<'a, S, D> Copy for Segment<'a, S, D>
where
    S: SourceLines<'a> + ?Sized,
    D: Spanned,
    <S::Line as LineSegmented<'a, S>>::View: Copy,
    D: Copy,
{
}

impl<'a, S, D> Clone for Segment<'a, S, D>
where
    S: SourceLines<'a> + ?Sized,
    D: Spanned,
    <S::Line as LineSegmented<'a, S>>::View: Clone,
    D: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let &Self {
            line_view: ref line,
            ref data,
        } = self;

        let line = line.clone();
        let data = data.clone();

        Self { line_view: line, data }
    }
}

impl<'a, S, D> fmt::Debug for Segment<'a, S, D>
where
    S: SourceLines<'a> + ?Sized,
    D: Spanned,
    <S::Line as LineSegmented<'a, S>>::View: fmt::Debug,
    D: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { line_view, data } = self;

        f.debug_struct("Segment").field("line", line_view).field("span", data).finish()
    }
}

/// An irrelevant line segment.
///
/// This is quite literally [`Segment`] but with a hardcoded [`Span`] as its
/// associated data.
pub type IrrelevantSegment<'a, S> = Segment<'a, S, Span>;
