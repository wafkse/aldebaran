//! Projection of spanned data onto individual source lines.
//!
//! [`LineSegmented`] maps arbitrary spanned values into line-local views while
//! retaining the original data. Segment iterators may preserve, split, or omit
//! inputs according to source intersection without imposing an input ordering.

pub mod iter;
pub mod piecewise;

pub use piecewise::Piecewise;

pub use iter::Segmented;

use core::{cmp, fmt, hash, iter::FusedIterator};

use aldebaran_span::prelude::{Span, Spanned};

use crate::line::{LineId, SourceLines};

/// Trait for [`lines`](SourceLines::Line) that have been segmented off a
/// source.
pub trait LineSegmented<'source, S>
where
    S: SourceLines<'source, Line = Self> + ?Sized,
{
    /// The iterator type produced for segmented input data.
    ///
    /// As noted in [`LineSegmented::segmented`], there is no implied order in
    /// the segments shall be given. Thus, users must sort the segments if they
    /// require some order to be present.
    type Segments<'input, I, D>: Iterator<Item = Piecewise<'source, S, D>>
    where
        I: Iterator<Item = D> + FusedIterator,
        S: 'input,
        D: Spanned,
        'source: 'input;

    /// A view into a source line.
    ///
    /// This is provided as a convenience for higher-kinded abstraction.
    ///
    /// Thus, in most cases, this will be the same as the source line.
    ///
    /// Also, in a generic context, this serves as a 'token type' for a view
    /// into the source line, but not the source line itself.
    type View;

    /// Segment the source into lines that intersect with the target segments.
    ///
    /// *No order* is imposed on the input [`spans`](Span), and thus the
    /// segments can be in any order.
    ///
    /// ## On the `None` return value
    ///
    /// The discriminant of the return values represents if the source can be
    /// segmented or not. This is the case for empty sources.
    ///
    /// If a span from the slice does not intersect with the source, it is
    /// ignored.
    fn segmented<'input, I, D>(line: &'input LineId<'source, S>, data_iter: I) -> Self::Segments<'input, I, D>
    where
        'source: 'input,
        I: Iterator<Item = D> + FusedIterator,
        D: Spanned;

    /// Retrieve a view into the source line for the given span.
    fn window(&self, span: Span) -> Option<Self::View>;
}

/// A single relevant line segment.
///
/// This encapsulates a view into the source line with some data `D`.
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
    #[inline]
    pub const fn data(&self) -> &D {
        let &Self { ref data, .. } = self;

        data
    }

    /// Construct a new [`Segment`] from its raw parts: the line view and the
    /// data.
    #[inline]
    pub const fn from_raw_parts(line_view: <S::Line as LineSegmented<'a, S>>::View, data: D) -> Self {
        Self { line_view, data }
    }

    /// Deconstruct this segment into its raw parts: the line view and the data.
    #[inline]
    pub fn into_raw_parts(self) -> (<S::Line as LineSegmented<'a, S>>::View, D) {
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
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
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
            cmp::Ordering::Equal => data.cmp(&other_data),
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
            Some(cmp::Ordering::Equal) => data.partial_cmp(&other_data),
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
        f.debug_struct("Segment")
            .field("line", &self.line_view)
            .field("span", &self.data)
            .finish()
    }
}

/// An irrelevant line segment.
///
/// This is quite literally [`Segment`] but with a hardcoded [`Span`] as its associated data.
pub type IrrelevantSegment<'a, S> = Segment<'a, S, Span>;
