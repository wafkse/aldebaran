//! The module for the [`Segmented`](Segmented) iterator adaptor.
//!
//! Refer to its documentation for more information.

use alloc::collections::BTreeMap;

use aldebaran_ice::Ice;

use core::{cmp::Ordering, fmt, hash, iter::FusedIterator, marker::PhantomData, num::NonZero, ops::Bound};

use aldebaran_span::prelude::{Span, Spanned};

use crate::line::{LineContent, LineId, LineSegmented, Piecewise, Segment, SourceLines};

/// An iterator adaptor over the segments of a line.
///
/// This adaptor adapts a [`Span`]-based iterator into a [`Piecewise`] iterator.
pub struct Segmented<'source, 'borrow, S, I, D>
where
    S: SourceLines<'source> + ?Sized,
    I: Iterator<Item = D> + FusedIterator,
    D: Spanned,
{
    /// The target line to be segmented.
    line: &'borrow LineId<'source, S>,

    /// The storage B-tree corresponding to the line's relevant spans.
    spans: BTreeMap<usize, D>,

    /// The already-revised span for this line.
    ///
    /// This serves as a context to determine whether the current iteration
    /// warrants a [`relevant`] or an
    /// [`irrelevant`] segment.
    target_index: usize,

    /// Mark the iterator type as used.
    _marker: PhantomData<I>,
}

impl<'source, 'borrow, S, I, D> Iterator for Segmented<'source, 'borrow, S, I, D>
where
    S: SourceLines<'source> + ?Sized,
    I: Iterator<Item = D> + FusedIterator,
    D: Spanned,
{
    type Item = Piecewise<'source, S, D>;

    fn next(&mut self) -> Option<Self::Item> {
        let &mut Self {
            line,
            ref mut spans,
            target_index: start_index,
            ..
        } = self;

        let line_content_len = line.content()?.span().length().get();

        let mut target_range = start_index..line_content_len;

        if target_range.is_empty() {
            return None;
        }

        loop {
            let target_value = target_range.next().map(|index| (index, spans.remove(&index)));

            match target_value {
                Some((char_index, Some(data))) => {
                    let span = data.span();

                    match start_index.cmp(&char_index) {
                        Ordering::Equal => {
                            let line_view = line.content().map(LineContent::value)?.window(span)?;

                            let target_segment = Segment { line_view, data };

                            // NOTE: to support overlapping spans, set the index
                            // to the next
                            // span's start that overlaps with the current span,
                            // or default to the end
                            // of the current span.
                            let span_end = char_index + span.length().get();

                            self.target_index = spans
                                .range((Bound::Excluded(char_index), Bound::Included(span_end)))
                                .map(|(index, _)| *index)
                                .fold(span_end, |acc, index| acc.min(index));

                            break Some(Piecewise::Relevant(target_segment));
                        }
                        Ordering::Less => {
                            let relevant_span = Span::new(start_index, NonZero::new(char_index - start_index)?);

                            let line_view = line.content().map(LineContent::value)?.window(relevant_span)?;

                            let target_segment = Segment {
                                line_view,
                                data: relevant_span,
                            };

                            self.target_index = char_index;

                            break Some(Piecewise::Irrelevant(target_segment));
                        }
                        Ordering::Greater => Ice::<()>::raise(),
                    }
                }
                None => {
                    let irrelevant_span = Span::new(start_index, NonZero::new(line_content_len - start_index)?);

                    let line_view = line.content().map(LineContent::value)?.window(irrelevant_span)?;

                    let target_segment = Segment {
                        line_view,
                        data: irrelevant_span,
                    };

                    // No more line to iterate over.
                    self.target_index = line_content_len;

                    break Some(Piecewise::Irrelevant(target_segment));
                }
                Some((_, None)) => { /* eat anything deemed irrelevant */ }
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
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let &Self { line, ref spans, .. } = self;

        line.hash(state);
        spans.hash(state);
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
    fn eq(&self, other: &Self) -> bool {
        self.line == other.line && self.spans == other.spans
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
    #[inline]
    pub fn new(line: &'borrow LineId<'source, S>, spans: I) -> Self {
        let line_span = line.span();

        let spans = spans
            .filter(|d0| d0.span().overlaps(line_span))
            .map(|d0| (d0.span().start(), d0))
            .collect::<BTreeMap<_, _>>();

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
