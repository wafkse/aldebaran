//! Incremental line discovery over iterable sources.
//!
//! [`LineIter`] recognizes line breaks while traversing source components and
//! assigns absolute source-coordinate spans to yielded [`LineId`] values.

use crate::dissect::SourceDissect;

use core::{cmp::Ordering, fmt, num::NonZero};

use aldebaran_baseline::prelude::{Span, Spanned};

use crate::{
    component::Component,
    iter::SourceIter,
    line::{LineContent, LineId, SourceLines},
    terminate::{Outcome, Terminator, TerminatorContext},
};

/// Numbered line discovery with absolute spans under a [`Terminator`] policy.
///
/// Empty input has no lines, and a trailing break adds no empty final line.
/// A blank terminated line has a break view without a content view. Numbering
/// starts at one even when traversal begins at a nonzero source offset, unless
/// a number is supplied through [`raw`](Self::raw).
// NOTE(invariant): Terminator = StateBefore(NextLine.TargetOffset)
pub struct LineIter<'a, S, T>
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + SourceDissect<'a, View<'a> = &'a S> + ?Sized,
    T: Terminator<'a, S>,
{
    /// Original source whose coordinate plane contains every yielded span.
    pub(in crate::line) target_slice: &'a S,

    /// The source coordinate where the next line begins.
    pub(in crate::line) target_offset: usize,

    /// The number assigned to the next yielded line.
    pub(in crate::line) target_index: NonZero<usize>,

    /// Recognition policy with state corresponding to the next line's start.
    pub(in crate::line) terminator: T,
}

impl<'a, S, T> Ord for LineIter<'a, S, T>
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + SourceDissect<'a, View<'a> = &'a S> + ?Sized,
    T: Terminator<'a, S>,
    S: Ord,
    T: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        let Self {
            target_slice,
            target_offset,
            target_index,
            terminator,
            ..
        } = self;

        let Self {
            target_slice: other_target_slice,
            target_offset: other_target_offset,
            target_index: other_target_index,
            terminator: other_terminator,
            ..
        } = other;

        match target_slice.cmp(other_target_slice) {
            Ordering::Equal => match target_offset.cmp(other_target_offset) {
                Ordering::Equal => match target_index.cmp(other_target_index) {
                    Ordering::Equal => terminator.cmp(other_terminator),
                    target_order => target_order,
                },
                target_order => target_order,
            },
            target_order => target_order,
        }
    }
}

impl<'a, S, T> PartialOrd for LineIter<'a, S, T>
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + SourceDissect<'a, View<'a> = &'a S> + ?Sized,
    T: Terminator<'a, S>,
    S: PartialOrd,
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let Self {
            target_slice,
            target_offset,
            target_index,
            terminator,
            ..
        } = self;

        let Self {
            target_slice: other_target_slice,
            target_offset: other_target_offset,
            target_index: other_target_index,
            terminator: other_terminator,
            ..
        } = other;

        match target_slice.partial_cmp(other_target_slice) {
            Some(Ordering::Equal) => match target_offset.partial_cmp(other_target_offset) {
                Some(Ordering::Equal) => match target_index.partial_cmp(other_target_index) {
                    Some(Ordering::Equal) => terminator.partial_cmp(other_terminator),
                    target_order => target_order,
                },
                target_order => target_order,
            },
            target_order => target_order,
        }
    }
}

impl<'a, S, T> Eq for LineIter<'a, S, T>
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + SourceDissect<'a, View<'a> = &'a S> + ?Sized,
    T: Terminator<'a, S>,
    S: Eq,
    T: Eq,
{
}

impl<'a, S, T> PartialEq for LineIter<'a, S, T>
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + SourceDissect<'a, View<'a> = &'a S> + ?Sized,
    T: Terminator<'a, S>,
    S: PartialEq,
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self {
            target_slice,
            target_offset,
            target_index,
            terminator,
            ..
        } = self;

        let Self {
            target_slice: other_target_slice,
            target_offset: other_target_offset,
            target_index: other_target_index,
            terminator: other_terminator,
            ..
        } = other;

        target_slice == other_target_slice
            && target_offset == other_target_offset
            && target_index == other_target_index
            && terminator == other_terminator
    }
}

impl<'a, S, T> fmt::Debug for LineIter<'a, S, T>
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + SourceDissect<'a, View<'a> = &'a S> + ?Sized,
    T: Terminator<'a, S>,
    S: fmt::Debug,
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            target_slice,
            target_offset,
            target_index,
            terminator,
            ..
        } = self;

        f.debug_struct("LineIter")
            .field("target_slice", target_slice)
            .field("target_offset", target_offset)
            .field("target_index", target_index)
            .field("terminator", terminator)
            .finish()
    }
}

impl<'a, S, T> Iterator for LineIter<'a, S, T>
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + SourceDissect<'a, View<'a> = &'a S> + ?Sized,
    T: Terminator<'a, S>,
{
    type Item = LineId<'a, S>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let &mut Self {
            target_slice,
            ref mut target_offset,
            ref mut target_index,
            ref mut terminator,
        } = self;

        let line_index = *target_index;
        let line_offset = *target_offset;

        let target_view = target_slice.offset(line_offset)?;

        let line_id: Option<LineId<'_, S>> = {
            let mut target_iter = target_view.enumerate();

            loop {
                if let Some((component_index, target_component)) = target_iter.next() {
                    let ctx = TerminatorContext::raw(
                        target_slice,
                        target_component,
                        Span::new(line_offset + component_index, target_component.size()),
                    );

                    let outcome = terminator.state(&ctx);

                    match outcome {
                        Outcome::Continue(_) => continue,
                        Outcome::Break(target_break) => {
                            let sequence_end = component_index + target_component.size().get();

                            let line_length = NonZero::new(sequence_end.saturating_sub(target_break.span().length().get()));

                            let line_span = line_length.map(|length| Span::new(line_offset, length));

                            let line_id = if let Some(line_span) = line_span {
                                let target_line = target_slice.dissect(line_span);

                                let line_content = LineContent::raw(target_line, line_span);

                                LineId::full(line_index, line_content, target_break)
                            } else {
                                LineId::break_only(line_index, target_break)
                            };

                            break Some(line_id);
                        }
                    }
                } else {
                    break match target_view.footprint() {
                        Some(target_footprint) => {
                            let line_span = Span::new(line_offset, target_footprint.length());
                            let target_line = target_slice.dissect(line_span);
                            let line_content = LineContent::raw(target_line, line_span);

                            Some(LineId::standalone(line_index, line_content))
                        }
                        None => None,
                    };
                }
            }
        };

        if let Some(ref line_id) = line_id {
            *target_offset += line_id.span().length().get();
            *target_index = line_index.saturating_add(1);
        }

        line_id
    }
}

impl<'a, S, T> LineIter<'a, S, T>
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + SourceDissect<'a, View<'a> = &'a S> + ?Sized,
    T: Terminator<'a, S>,
{
    /// Iterate from source coordinate zero using `T`'s
    /// [`Default::default`](core::default::Default::default) value.
    #[inline]
    pub fn new(target_source: &'a S) -> Self
    where
        T: Default,
    {
        Self::new_with(target_source, T::default())
    }

    /// Treat the source suffix at `target_offset` as the start of line one.
    ///
    /// Uses `T`'s default recognition state. String offsets must be UTF-8
    /// boundaries. A position at or beyond the source end yields no lines.
    #[inline]
    pub fn new_at(target_source: &'a S, target_offset: usize) -> Self
    where
        T: Default,
    {
        Self::new_at_with(target_source, target_offset, T::default())
    }

    /// Iterate from the source's first coordinate using `terminator`.
    #[inline]
    pub const fn new_with(target_source: &'a S, terminator: T) -> Self {
        let target_slice = target_source;

        let target_offset = 0;

        let target_index = NonZero::<usize>::MIN;

        Self {
            target_slice,
            target_offset,
            target_index,
            terminator,
        }
    }

    /// Treat a source suffix as line one while retaining absolute coordinates.
    ///
    /// If `target_offset` falls inside a line, its preceding content is omitted.
    /// String offsets must be UTF-8 boundaries, and `terminator` must carry
    /// suitable recognition state for the chosen start. At or beyond the source
    /// end, traversal yields no lines.
    #[inline]
    pub const fn new_at_with(target_source: &'a S, target_offset: usize, terminator: T) -> Self {
        let target_slice = target_source;

        let target_index = NonZero::<usize>::MIN;

        Self {
            target_slice,
            target_offset,
            target_index,
            terminator,
        }
    }

    /// Resume at a caller-supplied source position and line number.
    ///
    /// The offset must be a valid source boundary and the terminator state
    /// must correspond to that position. The supplied number labels the first
    /// yielded line rather than being derived from preceding source content.
    #[inline]
    pub const fn raw(target_slice: &'a S, target_offset: usize, target_index: NonZero<usize>, terminator: T) -> Self {
        Self {
            target_slice,
            target_offset,
            target_index,
            terminator,
        }
    }

    /// Label reserved for the next line, saturating at `usize::MAX`.
    #[inline]
    pub const fn index(&self) -> NonZero<usize> {
        let &Self { target_index, .. } = self;

        target_index
    }

    /// Absolute start of the remaining suffix in source units.
    /// At exhaustion this is the source end for a traversal begun within bounds.
    #[inline]
    pub const fn offset(&self) -> usize {
        let &Self { target_offset, .. } = self;

        target_offset
    }

    /// Original source backing the remaining suffix and all yielded line views.
    #[inline]
    pub const fn source(&self) -> &'a S {
        let &Self { target_slice, .. } = self;

        target_slice
    }
}

impl<'a, S, T> Clone for LineIter<'a, S, T>
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + SourceDissect<'a, View<'a> = &'a S> + ?Sized,
    T: Terminator<'a, S>,
    T: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, S, T> Copy for LineIter<'a, S, T>
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + SourceDissect<'a, View<'a> = &'a S> + ?Sized,
    T: Terminator<'a, S>,
    T: Copy,
{
}

#[cfg(test)]
mod tests {
    use aldebaran_baseline::span::Spanned;

    use crate::terminate::Standard;

    use super::LineIter;

    #[test]
    fn empty_sources_do_not_create_a_line_or_advance() {
        let mut text = LineIter::<str, Standard<char>>::new("");
        let mut bytes = LineIter::<[u8], Standard<u8>>::new(b"");

        for _ in 0..3 {
            assert!(text.next().is_none(), "an empty string has no source lines");
            assert!(bytes.next().is_none(), "an empty byte slice has no source lines");
            assert_eq!(text.offset(), 0);
            assert_eq!(text.index().get(), 1);
            assert_eq!(bytes.offset(), 0);
            assert_eq!(bytes.index().get(), 1);
        }
    }

    #[test]
    fn utf8_lines_preserve_absolute_content_and_break_extents() {
        let source = "α\r\n\nβ";
        let mut lines = LineIter::<str, Standard<char>>::new(source);
        let first = lines.next().expect("the first line contains alpha and CRLF");

        assert_eq!(first.number().get(), 1);
        assert_eq!(first.span().range(), 0..4);
        assert_eq!(first.content().map(|content| *content.value()), Some("α"));
        assert_eq!(first.content().map(|content| content.span().range()), Some(0..2));
        assert_eq!(first.content_break().map(|line_break| *line_break.line_break()), Some("\r\n"));
        assert_eq!(first.content_break().map(|line_break| line_break.span().range()), Some(2..4));

        let blank = lines.next().expect("the second line is a bare LF");

        assert_eq!(blank.number().get(), 2);
        assert_eq!(blank.span().range(), 4..5);
        assert!(blank.content().is_none(), "a break-only line has no content extent");
        assert_eq!(blank.content_break().map(|line_break| *line_break.line_break()), Some("\n"));

        let last = lines.next().expect("the final line has no terminator");

        assert_eq!(last.number().get(), 3);
        assert_eq!(last.span().range(), 5..7);
        assert_eq!(last.content().map(|content| *content.value()), Some("β"));
        assert!(last.content_break().is_none(), "the final source line is unterminated");

        for _ in 0..3 {
            assert!(lines.next().is_none(), "exhausted line iteration must remain exhausted");
            assert_eq!(lines.offset(), source.len());
            assert_eq!(lines.index().get(), 4);
        }
    }

    #[test]
    fn trailing_break_does_not_create_an_empty_final_line() {
        let mut lines = LineIter::<[u8], Standard<u8>>::new(b"a\r\n");
        let first = lines.next().expect("the source contains a terminated line");

        assert_eq!(first.content().map(|content| *content.value()), Some(b"a".as_slice()));
        assert_eq!(
            first.content_break().map(|line_break| *line_break.line_break()),
            Some(b"\r\n".as_slice())
        );
        assert!(lines.next().is_none(), "a terminal break does not add source content");
        assert_eq!(lines.offset(), 3);
    }

    #[test]
    fn starting_inside_a_line_retains_absolute_coordinates() {
        let mut lines = LineIter::<str, Standard<char>>::new_at("ab\nβ", 1);
        let first = lines.next().expect("the remaining first line contains b");

        assert_eq!(first.number().get(), 1);
        assert_eq!(first.content().map(|content| *content.value()), Some("b"));
        assert_eq!(first.span().range(), 1..3);

        let second = lines.next().expect("the second line contains beta");

        assert_eq!(second.number().get(), 2);
        assert_eq!(second.content().map(|content| *content.value()), Some("β"));
        assert_eq!(second.span().range(), 3..5);
        assert!(lines.next().is_none(), "both remaining lines have been consumed");
        assert_eq!(lines.offset(), 5);
    }

    #[test]
    fn repeated_carriage_return_can_start_a_crlf_break() {
        let mut lines = LineIter::<str, Standard<char>>::new("a\r\r\nb");
        let first = lines.next().expect("the first line ends in CRLF");

        assert_eq!(first.content().map(|content| *content.value()), Some("a\r"));
        assert_eq!(first.content_break().map(|line_break| *line_break.line_break()), Some("\r\n"));
        assert_eq!(first.content_break().map(|line_break| line_break.span().range()), Some(2..4));

        let second = lines.next().expect("the final line contains b");

        assert_eq!(second.content().map(|content| *content.value()), Some("b"));
        assert_eq!(second.span().range(), 4..5);
    }
}
