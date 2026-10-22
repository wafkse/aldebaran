//! Projection of spanned data onto individual source lines.
//!
//! [`LineSegmented`] maps spanned values onto source-specific line views while
//! retaining their associated data. Since callers may supply unordered input,
//! each implementation defines how overlapping and duplicate spans appear in
//! the resulting segments.

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
    /// Annotation-bearing windows and unannotated gaps produced for a line.
    type Segments<'input, I, D>: Iterator<Item = Piecewise<'source, S, D>>
    where
        I: Iterator<Item = D> + FusedIterator,
        S: 'input,
        D: Spanned,
        'source: 'input;

    /// The representation returned for a window of the source line.
    type View;

    /// Project annotations expressed in the original source coordinates onto this line.
    ///
    /// Input need not be ordered. Implementations define how overlapping or
    /// duplicate spans are represented.
    fn segmented<'input, I, D>(line: &'input LineId<'source, S>, data_iter: I) -> Self::Segments<'input, I, D>
    where
        'source: 'input,
        I: Iterator<Item = D> + FusedIterator,
        D: Spanned;

    /// Select a window using coordinates relative to this line view.
    ///
    /// `None` means the window is out of bounds or not representable. For
    /// string views both endpoints must be UTF-8 boundaries.
    fn window(&self, span: Span) -> Option<Self::View>;
}

/// A line window retaining the annotation that selected it.
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
    /// Renderable window within the associated data's source extent.
    line_view: <S::Line as LineSegmented<'a, S>>::View,

    /// Original annotation data, retaining its unclipped source span.
    data: D,
}

impl<'a, S, D> Segment<'a, S, D>
where
    S: SourceLines<'a> + ?Sized,
    D: Spanned,
{
    /// Window available for rendering, which may cover only part of the annotation.
    #[inline]
    pub const fn view(&self) -> &<S::Line as LineSegmented<'a, S>>::View {
        let &Self { ref line_view, .. } = self;

        line_view
    }

    /// Original annotation with its source span preserved across clipping.
    #[inline]
    pub const fn data(&self) -> &D {
        let &Self { ref data, .. } = self;

        data
    }

    /// Associate a line window with data covering its source region.
    /// The caller must keep the view within the data's span in the original source.
    #[inline]
    pub const fn raw(line_view: <S::Line as LineSegmented<'a, S>>::View, data: D) -> Self {
        Self { line_view, data }
    }

    /// Separate the renderable window from its original annotation.
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

/// A line interval that represents a gap in caller-provided annotations.
pub type IrrelevantSegment<'a, S> = Segment<'a, S, Span>;
