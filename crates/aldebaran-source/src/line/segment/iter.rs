//! The module for the [`Segmented`](Segmented) iterator adaptor.
//!
//! See [`Segmented`] for further information.

use alloc::{collections::VecDeque, vec::Vec};

use core::{fmt, hash, iter::FusedIterator, marker::PhantomData, num::NonZero};

use aldebaran_baseline::prelude::{Span, Spanned};

use crate::line::{LineId, LineSegmented, Piecewise, Segment, SourceLines};

/// An iterator adaptor over the segments of a line.
///
/// This adaptor adapts a [`Span`]-based iterator into a [`Piecewise`] iterator.
/// Construction consumes the annotation iterator and allocates storage for
/// representable content overlaps. Views are ordered by their clipped start,
/// with input order preserved for equal starts. Overlapping annotations remain
/// separate and retain their original data and spans, so the output is not a
/// disjoint partition. Gaps cover only content outside their combined coverage.
/// A line without content yields no pieces.
// NOTE(invariant): ValidWindows(Pending, Line.Content) And SortedByStart(Pending)
// And TargetIndex = Max({0} Union LocalEndpoints(Emitted))
pub struct Segmented<'source, 'borrow, S, I, D>
where
    S: SourceLines<'source> + ?Sized,
    I: Iterator<Item = D> + FusedIterator,
    D: Spanned,
{
    /// The target line to be segmented.
    line: &'borrow LineId<'source, S>,

    /// Pending local windows paired with original annotations, ordered by local start.
    spans: VecDeque<(Span, D)>,

    /// Greatest emitted endpoint relative to the content start, used to bound gaps.
    target_index: usize,

    /// Input iterator identity retained after its annotations have been collected.
    _marker: PhantomData<I>,
}

impl<'source, 'borrow, S, I, D> Iterator for Segmented<'source, 'borrow, S, I, D>
where
    S: SourceLines<'source> + ?Sized,
    I: Iterator<Item = D> + FusedIterator,
    D: Spanned,
{
    type Item = Piecewise<'source, S, D>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let &mut Self {
            line,
            ref mut spans,
            target_index: ref mut next_index,
            ..
        } = self;

        let content = line.content()?;
        let content_span = content.span();
        let line_content_len = content_span.length().get();

        match spans.front() {
            Some((span, _)) if span.start() > *next_index => {
                let length = NonZero::new(span.start() - *next_index)?;
                let local = Span::new(*next_index, length);
                let line_view = content.value().window(local)?;
                let data = Span::new(content_span.start() + *next_index, length);

                *next_index = span.start();

                Some(Piecewise::Irrelevant(Segment { line_view, data }))
            }
            Some(_) => {
                let (span, data) = spans.pop_front()?;

                let line_view = content.value().window(span)?;

                *next_index = (*next_index).max(span.end());

                Some(Piecewise::Relevant(Segment { line_view, data }))
            }
            None => {
                let length = NonZero::new(line_content_len - *next_index)?;
                let local = Span::new(*next_index, length);
                let line_view = content.value().window(local)?;
                let data = Span::new(content_span.start() + *next_index, length);

                *next_index = line_content_len;

                Some(Piecewise::Irrelevant(Segment { line_view, data }))
            }
        }
    }
}

impl<'source, 'borrow, S, I, D> hash::Hash for Segmented<'source, 'borrow, S, I, D>
where
    S: SourceLines<'source> + ?Sized,
    I: Iterator<Item = D> + FusedIterator,
    D: Spanned,
    S::Line: hash::Hash,
    I: hash::Hash,
    D: hash::Hash,
{
    #[inline]
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
        let &Self {
            line,
            ref spans,
            target_index,
            ..
        } = self;

        line.hash(state);
        spans.hash(state);
        target_index.hash(state);
    }
}

impl<'source, 'borrow, S, I, D> Eq for Segmented<'source, 'borrow, S, I, D>
where
    S: SourceLines<'source> + ?Sized,
    I: Iterator<Item = D> + FusedIterator,
    D: Spanned,
    S::Line: Eq,
    I: Eq,
    D: Eq,
{
}

impl<'source, 'borrow, S, I, D> PartialEq for Segmented<'source, 'borrow, S, I, D>
where
    S: SourceLines<'source> + ?Sized,
    I: Iterator<Item = D> + FusedIterator,
    D: Spanned,
    S::Line: PartialEq,
    I: PartialEq,
    D: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let Self {
            line, spans, target_index, ..
        } = self;

        let Self {
            line: other_line,
            spans: other_spans,
            target_index: other_index,
            ..
        } = other;

        line == other_line && spans == other_spans && target_index == other_index
    }
}

impl<'source, 'borrow, S, I, D> Clone for Segmented<'source, 'borrow, S, I, D>
where
    S: SourceLines<'source> + ?Sized,
    I: Iterator<Item = D> + FusedIterator,
    D: Spanned,
    S::Line: Clone,
    D: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let &Self {
            line,
            ref spans,
            target_index,
            _marker,
        } = self;

        let spans = spans.clone();

        Self {
            line,
            spans,
            target_index,

            _marker,
        }
    }
}

impl<'source, 'borrow, S, I, D> fmt::Debug for Segmented<'source, 'borrow, S, I, D>
where
    S: SourceLines<'source> + ?Sized,
    I: Iterator<Item = D> + FusedIterator,
    D: Spanned,
    S::Line: fmt::Debug,
    I: fmt::Debug,
    D: fmt::Debug,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let &Self {
            line,
            ref spans,
            ref target_index,
            ..
        } = self;

        f.debug_struct("Segmented")
            .field("line", line)
            .field("spans", spans)
            .field("target_index", target_index)
            .finish()
    }
}

impl<'source, 'borrow, S, I, D> Segmented<'source, 'borrow, S, I, D>
where
    S: SourceLines<'source> + ?Sized,
    I: Iterator<Item = D> + FusedIterator,
    D: Spanned,
{
    /// Construct a new [`Segmented`] iterator.
    ///
    /// Views are clipped to content while data remains intact. An annotation
    /// is omitted if it has no content overlap or if that overlap cannot be
    /// represented by [`LineSegmented::window`], including invalid UTF-8 endpoints.
    /// The input is consumed eagerly when the line has content.
    #[inline]
    pub fn new(line: &'borrow LineId<'source, S>, spans: I) -> Self {
        let mut spans = match line.content() {
            Some(content) => spans
                .filter_map(|data| {
                    let clipped = data.span().intersect(content.span())?;
                    let local = Span::new(clipped.start() - content.span().start(), clipped.length());

                    content.value().window(local)?;

                    Some((local, data))
                })
                .collect::<Vec<_>>(),
            None => Vec::new(),
        };

        spans.sort_by_key(|(span, _)| span.start());

        let spans = spans.into();

        let target_index = 0;

        let _marker = PhantomData;

        Self {
            line,
            spans,
            target_index,
            _marker,
        }
    }
}

#[cfg(test)]
mod tests {
    //! Tests module for the [`iter`] module.
    //!
    //! [`iter`]: super

    //! Annotation clipping, overlap ordering, and gap-coordinate expectations.

    use alloc::vec::Vec;
    use core::num::NonZero;

    use aldebaran_baseline::prelude::{Span, Spanned};

    use crate::line::{LineExtent, LineSegmented, SourceLines};

    /// Nonempty source interval specified by an inclusive start and exclusive end.
    fn span(start: usize, end: usize) -> Span {
        Span::new(start, NonZero::new(end - start).expect("nonempty test interval"))
    }

    #[test]
    fn later_lines_clip_crossing_spans_and_exclude_terminators() {
        let source = "first\r\nsecond\r\n";
        let extents = source.extents().expect("source lines");
        let line = extents.lines().nth(1).expect("second line");
        let annotations = [span(5, 8), span(11, 15), span(13, 15)];
        let pieces: Vec<_> = LineSegmented::segmented(&line, annotations.into_iter()).collect();

        assert_eq!(pieces.iter().map(|piece| *piece.view()).collect::<Vec<_>>(), ["s", "eco", "nd"]);
        assert_eq!(
            pieces.iter().map(Spanned::span).collect::<Vec<_>>(),
            [span(5, 8), span(8, 11), span(11, 15)]
        );
        assert_eq!(
            pieces.iter().filter_map(|piece| piece.data().copied()).collect::<Vec<_>>(),
            annotations[..2]
        );
    }

    #[test]
    fn gaps_preserve_pending_annotations_and_overlaps_cover_their_union() {
        let source = "0123456789abcdef";
        let extents = source.extents().expect("source lines");
        let line = extents.lines().next().expect("first line");
        let annotations = [span(4, 12), span(4, 8), span(6, 7), span(10, 14)];
        let mut segments = LineSegmented::segmented(&line, annotations.into_iter());
        let pieces: Vec<_> = segments.by_ref().collect();

        assert_eq!(
            pieces.iter().map(|piece| *piece.view()).collect::<Vec<_>>(),
            ["0123", "456789ab", "4567", "6", "abcd", "ef"]
        );
        assert_eq!(
            pieces.iter().filter_map(|piece| piece.data().copied()).collect::<Vec<_>>(),
            annotations
        );
        assert_eq!(pieces.last().map(Spanned::span), Some(span(14, 16)));
        assert!(segments.next().is_none());
        assert!(segments.next().is_none());
    }

    #[test]
    fn unordered_byte_annotations_keep_duplicate_starts_after_clipping() {
        let source = b"first\nsecond".as_slice();
        let extents = source.extents().expect("source lines");
        let line = extents.lines().nth(1).expect("second line");
        let annotations = [span(9, 10), span(6, 8), span(5, 7)];
        let pieces: Vec<_> = LineSegmented::segmented(&line, annotations.into_iter()).collect();

        assert_eq!(
            pieces.iter().map(|piece| *piece.view()).collect::<Vec<_>>(),
            [b"se".as_slice(), b"s", b"c", b"o", b"nd"]
        );
        assert_eq!(
            pieces.iter().filter_map(|piece| piece.data().copied()).collect::<Vec<_>>(),
            [annotations[1], annotations[2], annotations[0]]
        );
    }

    #[test]
    fn invalid_utf8_windows_do_not_hide_later_annotations() {
        let source = "é中x";
        let extents = source.extents().expect("source lines");
        let line = extents.lines().next().expect("first line");
        let annotations = [Span::unit(1), span(2, 5), Span::unit(5)];
        let pieces: Vec<_> = LineSegmented::segmented(&line, annotations.into_iter()).collect();

        assert_eq!(pieces.iter().map(|piece| *piece.view()).collect::<Vec<_>>(), ["é", "中", "x"]);
        assert_eq!(
            pieces.iter().filter_map(|piece| piece.data().copied()).collect::<Vec<_>>(),
            annotations[1..]
        );
    }
}
