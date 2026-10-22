//! Source lines as numbered views with absolute source spans.
//!
//! [`SourceLines`] selects a contiguous set of lines through a [`LineExtent`].
//! Its [`LineId`] values separate nonempty content from an optional terminator,
//! while [`Metadata`] describes the selected lines including their breaks.
//! [`LineSegmented`] projects annotations onto content for rendering without
//! including the terminators in those views.

mod content;
mod extent;
mod handle;
mod id;
mod iter;
mod line_break;
mod metadata;
pub mod preview;
mod segment;

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

use aldebaran_baseline::prelude::{Span, Spanned};

/// The line-extent type produced for a source and terminator.
pub type ExtentsOf<'a, S, T> = <S as SourceLines<'a>>::Extents<T>;

/// The line-extent type produced with a source's default terminator.
pub type DefaultExtents<'a, S> = ExtentsOf<'a, S, DefaultTerminator<'a, S>>;

/// Source-line selection under a configurable termination policy.
///
/// Selections retain the original source coordinates and line numbers, so a
/// renderer can inspect a subset without rebasing its annotations. The built-in
/// traversal yields no lines for empty input and adds no empty line after a
/// trailing terminator.
pub trait SourceLines<'a>: Source<'a> + SourceDissect<'a> + Terminated<'a, Self> {
    /// Content or terminator representation used by this source's line views.
    type Line: LineSegmented<'a, Self>;

    /// Traversal and aggregate metadata for a nonempty contiguous line selection.
    type Extents<T>: LineExtent<'a, Self>
    where
        T: Terminator<'a, Self>;

    /// Select every source line under [`Terminated::DEFAULT_TERMINATOR`].
    /// Empty input has no selection.
    #[inline]
    fn extents(&'a self) -> Option<Self::Extents<Self::TerminatedBy>> {
        self.extents_with(Self::DEFAULT_TERMINATOR)
    }

    /// Select every source line under a caller-selected termination policy.
    /// Empty input has no selection.
    #[inline]
    fn extents_with<T>(&'a self, terminator: T) -> Option<Self::Extents<T>>
    where
        T: Terminator<'a, Self>,
    {
        self.footprint().and_then(|span| self.segment_with::<T>(terminator, span))
    }

    /// Selects physical source lines using [`Terminated::DEFAULT_TERMINATOR`].
    ///
    /// See [`Self::segment_with`] for target selection and EOF behavior.
    #[inline]
    fn segment(&'a self, span: Span) -> Option<Self::Extents<Self::TerminatedBy>> {
        self.segment_with(Self::DEFAULT_TERMINATOR, span)
    }

    /// Selects physical source lines intersecting the target span.
    ///
    /// Line terminators participate in overlap checks.
    ///
    /// # Remarks
    ///
    /// A unit span at EOF selects the final physical line of nonempty input.
    ///
    /// This does not synthesize a trailing empty line.
    ///
    /// Empty input yields no selection. Other targets select only overlapping
    /// lines, including when part of the target extends beyond the source.
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

    #[expect(
        clippy::missing_inline_in_public_items,
        reason = "Line selection scans source components and assembles aggregate metadata."
    )]
    fn segment_with<T>(&'a self, terminator: T, relevant_span: Span) -> Option<Self::Extents<T>>
    where
        T: Terminator<'a, Self>,
    {
        let source_end = self.size();
        let is_eof = relevant_span == Span::unit(source_end);
        let is_relevant = |target_line: &LineId<'a, S>| {
            let span = target_line.span();

            span.overlaps(relevant_span) || (is_eof && span.end() == source_end)
        };

        let mut line_iter = LineIter::<'a, S, T>::new_with(self, terminator);

        // Resume with the state before the selected line, including for
        // terminators whose state persists across breaks.
        let (start_id, terminator) = loop {
            let terminator = line_iter.terminator;
            let line = line_iter.next()?;

            if is_relevant(&line) {
                break (line, terminator);
            }
        };

        let mut line_size_minmax = MinMax::initial(start_id.span().length().get());

        let end_id = line_iter
            .take_while(is_relevant)
            .inspect(|target_line| line_size_minmax.record(target_line.span().length().get()))
            .last();

        let end_id = end_id.as_ref().unwrap_or(&start_id);
        let line_range = LineRange::from_raw_parts(start_id.number(), end_id.number());
        let englobing_span = start_id.span().superset(end_id.span());

        let metadata = Metadata::from_raw_parts(englobing_span, line_range, line_size_minmax);

        Some(LineHandle::from_raw_parts(self, metadata, start_id, terminator))
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;
    use core::{num::NonZero, ops::Range};

    use aldebaran_baseline::prelude::{Span, Spanned};

    use super::{LineExtent, SourceLines};
    use crate::terminate::Standard;

    /// Check default and explicit policies against absolute extents and metadata.
    fn check_selection(source: &str, target: Span, expected: &[(usize, Range<usize>)]) {
        let selection = source.segment(target);
        let bytes = source.as_bytes().segment(target);
        let explicit = source.segment_with(Standard::<char>::new(), target);
        let explicit_bytes = source.as_bytes().segment_with(Standard::<u8>::new(), target);

        assert_eq!(selection, explicit);
        assert_eq!(bytes, explicit_bytes);

        if expected.is_empty() {
            assert!(selection.is_none(), "no string lines should be selected by {target}");
            assert!(bytes.is_none(), "no byte lines should be selected by {target}");
            return;
        }

        let selection = selection.expect("selected lines exist");
        let bytes = bytes.expect("selected byte lines exist");
        let mut lines = selection.lines();
        let actual: Vec<_> = lines.by_ref().map(|line| (line.number().get(), line.span().range())).collect();
        assert_eq!(actual, expected);
        assert!(lines.next().is_none(), "the selected iterator must stay exhausted");
        assert_eq!(
            bytes
                .lines()
                .map(|line| (line.number().get(), line.span().range()))
                .collect::<Vec<_>>(),
            expected
        );
        let (first_number, first_span) = expected.first().expect("nonempty selection");

        let (last_number, last_span) = expected.last().expect("nonempty selection");

        let lengths: Vec<_> = expected.iter().map(|(_, span)| span.len()).collect();
        let metadata = selection.metadata();
        assert_eq!(metadata.bytes().range(), first_span.start..last_span.end);
        assert_eq!(metadata.lines().range(), *first_number..=*last_number);
        assert_eq!(
            metadata.lines().into_iter().map(NonZero::get).collect::<Vec<_>>(),
            (*first_number..=*last_number).collect::<Vec<_>>()
        );
        assert_eq!(
            metadata.length().into_pair(),
            (
                *lengths.iter().min().expect("selected lengths"),
                *lengths.iter().max().expect("selected lengths")
            )
        );
        assert_eq!(metadata, bytes.metadata());
    }

    #[test]
    fn middle_adjacent_and_final_selections_have_exact_extents() {
        let source = "a\nbbb\r\nc\nlast";

        check_selection(source, Span::unit(3), &[(2, 2..7)]);
        check_selection(source, Span::new(3, NonZero::new(5).expect("nonzero")), &[(2, 2..7), (3, 7..9)]);
        check_selection(source, Span::unit(9), &[(4, 9..13)]);

        let selection = source.segment(Span::unit(6)).expect("CRLF is part of line two");
        let line = selection.lines().next().expect("selected line");
        assert_eq!(line.content().map(|content| *content.value()), Some("bbb"));
        assert_eq!(line.content_break().map(|line_break| *line_break.line_break()), Some("\r\n"));
    }

    #[test]
    fn all_small_spans_agree_with_overlap_or_exact_unit_eof_and_metadata() {
        for (source, full_lines) in [
            ("", &[][..]),
            ("abc", &[(1, 0..3)][..]),
            ("é", &[(1, 0..2)][..]),
            ("🦀", &[(1, 0..4)][..]),
            ("a\n", &[(1, 0..2)][..]),
            ("a\r\n", &[(1, 0..3)][..]),
            ("\n", &[(1, 0..1)][..]),
            ("\r\n", &[(1, 0..2)][..]),
            ("a\nbbb\r\nc\nlast", &[(1, 0..2), (2, 2..7), (3, 7..9), (4, 9..13)][..]),
            ("é\r\n\n🦀\n終", &[(1, 0..4), (2, 4..5), (3, 5..10), (4, 10..13)][..]),
            ("\r\n\n", &[(1, 0..2), (2, 2..3)][..]),
        ] {
            for start in 0..=source.len() + 1 {
                for length in 1..=source.len() + 2 {
                    let target = Span::new(start, NonZero::new(length).expect("positive length"));
                    let expected: Vec<_> = if start == source.len() && length == 1 {
                        full_lines.last().cloned().into_iter().collect()
                    } else {
                        full_lines
                            .iter()
                            .filter(|(_, span)| span.start < target.end() && span.end > start)
                            .cloned()
                            .collect()
                    };

                    check_selection(source, target, &expected);
                }
            }
            assert_eq!(source.extents().is_none(), source.is_empty());
        }
    }
}
