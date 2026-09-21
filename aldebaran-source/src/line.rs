//! Line discovery, segmentation, and metadata for source values.
//!
//! [`SourceLines`] layers line structure over iterable and dissectable sources.
//! Terminator-driven iterators produce absolute line identities, while extents and
//! segmentation expose only the lines and line-local pieces relevant to a span.

mod content;
mod extent;
mod handle;
mod id;
mod iter;
mod line_break;
mod metadata;
pub mod preview;
mod segment;

use core::num::NonZero;
use core::ops::Not;

use metadata::{LineRange, MinMax};

pub use self::{
    content::LineContent,
    extent::LineExtent,
    handle::LineHandle,
    id::LineId,
    iter::LineIter,
    line_break::LineBreak,
    metadata::Metadata,
    segment::{LineSegmented, Piecewise, Segment, Segmented},
};

use crate::source::{DefaultTerminator, Source};

use crate::iter::SourceIter;

use crate::terminate::Terminated;

use crate::dissect::SourceDissect;

use crate::terminate::Terminator;

use aldebaran_span::prelude::{Span, Spanned};

/// Determine the [`extents`] type of a source.
///
/// This is a very simple type alias that refers to the [`extents`] associated
/// type of the [`SourceLines`] trait.
///
/// If this is not used, the result will be inmmensely verbose.
///
/// [`extents`]: SourceLines::Extents
pub type ExtentsOf<'a, S, T> = <S as SourceLines<'a>>::Extents<T>;

/// Determine the default [`extents`] of a source.
///
/// This is exactly the same as [`DefaultTerminator`], but for extents.
///
/// [`extents`]: SourceLines::Extents
pub type DefaultExtents<'a, S> = ExtentsOf<'a, S, DefaultTerminator<'a, S>>;

/// A trait for source sequences that can be expressed as a linear sequence of
/// lines.
///
/// This is an **extension trait**, and thus only builds functionality on top of
/// [`Source`].
pub trait SourceLines<'a>: Source<'a> + SourceDissect<'a> + Terminated<'a, Self> {
    /// A potentially-borrowed line in the source.
    type Line: LineSegmented<'a, Self>;

    /// An associated type that represents the extents of one or more lines in
    /// the source.
    type Extents<T>: LineExtent<'a, Self>
    where
        T: Terminator<'a, Self>;

    /// Compute the [`extents`] for all lines in the source, using the default [`line terminator`].
    ///
    /// [`extents`]: SourceLines::Extents
    /// [`line terminator`]: Terminator
    #[inline]
    fn extents(&'a self) -> Option<Self::Extents<Self::TerminatedBy>> {
        self.extents_with(Self::DEFAULT_TERMINATOR)
    }

    /// Compute the [`extents`] for all lines in the source with the target [`line terminator`].
    ///
    /// [`extents`]: SourceLines::Extents
    /// [`line terminator`]: Terminator
    #[inline]
    fn extents_with<T>(&'a self, terminator: T) -> Option<Self::Extents<T>>
    where
        T: Terminator<'a, Self>,
    {
        self.footprint().map(|span| self.segment_with::<T>(terminator, span)).flatten()
    }

    /// Segment the source into lines that overlap the target [`span`].
    ///
    /// A unit span beginning at the exclusive source end does not overlap a
    /// source line and therefore produces [`None`]. Diagnostic renderers may
    /// interpret that span separately as a terminal insertion boundary.
    ///
    /// Uses the default [`terminator`] for the source.
    ///
    /// [`span`]: Span
    /// [`terminator`]: Terminator
    #[inline]
    fn segment(&'a self, span: Span) -> Option<Self::Extents<Self::TerminatedBy>> {
        self.segment_with(Self::DEFAULT_TERMINATOR, span)
    }

    ///  Segment the source into lines that coalesce with the target [`span`].
    ///
    /// [`span`]: Span
    /// [`extents`]: SourceLines::Extents
    /// [`line terminator`]: Terminator
    fn segment_with<T>(&'a self, terminator: T, span: Span) -> Option<Self::Extents<T>>
    where
        T: Terminator<'a, Self>;
}

impl<'a, S> SourceLines<'a> for S
where
    S: SourceIter<'a> + SourceDissect<'a, View<'a> = &'a S> + Terminated<'a, S> + ?Sized,
    &'a S: LineSegmented<'a, S>,
{
    type Line = &'a S;

    type Extents<T>
        = LineHandle<'a, S, T>
    where
        T: Terminator<'a, Self>;

    fn segment_with<T>(&'a self, terminator: T, relevant_span: Span) -> Option<Self::Extents<T>>
    where
        T: Terminator<'a, Self>,
    {
        let is_relevant = |target_line: &LineId<'a, S>| target_line.span().overlaps(relevant_span);

        let not_relevant = |target_line: &LineId<'a, S>| target_line.span().overlaps(relevant_span).not();

        let mut line_iter = LineIter::<'a, S, T>::new_with(self, terminator).skip_while(not_relevant);

        let start_id: LineId<'a, S> = line_iter.next()?;

        let mut line_size_minmax = MinMax::initial(start_id.span().length().get());

        let end_id: Option<LineId<'a, S>> = line_iter
            .map(|target_line: LineId<'_, _>| {
                let line_span: Span = target_line.span();

                line_size_minmax.record(line_span.length().get());

                target_line
            })
            .take_while(is_relevant)
            .last();

        let line_range = LineRange::from_raw_parts(start_id.number(), end_id.as_ref().map(LineId::number).unwrap_or(start_id.number()));

        let englobing_span = {
            match end_id.as_ref().map(LineId::span) {
                Some(end_span) => start_id.span().superset(end_span),
                None => {
                    let start_index = start_id.span().start();

                    Span::new(
                        start_index,
                        NonZero::new(self.size().saturating_sub(start_index)).unwrap_or(NonZero::<usize>::MIN),
                    )
                }
            }
        };

        let metadata = Metadata::from_raw_parts(englobing_span, line_range, line_size_minmax);

        Some(LineHandle::from_raw_parts(self, metadata, start_id, terminator))
    }
}
