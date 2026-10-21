//! Built-in annotation projection onto line content.
//!
//! [`Segmented`] clips renderable views to content, preserving the original
//! annotations in [`Piecewise::Relevant`] and filling uncovered intervals with
//! [`Piecewise::Irrelevant`]. Terminators are excluded from both kinds of view.

use alloc::{collections::VecDeque, vec::Vec};

use core::{fmt, hash, iter::FusedIterator, marker::PhantomData, num::NonZero};

use aldebaran_baseline::prelude::{Span, Spanned};

use crate::line::{LineId, LineSegmented, Piecewise, Segment, SourceLines};

/// Built-in projection of annotations and gaps onto a line view.
///
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
    /// Prepare annotations for rendering against this line's content.
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
mod tests;
