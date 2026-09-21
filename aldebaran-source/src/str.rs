//! Source capability implementations for borrowed UTF-8 strings.
//!
//! String sources iterate by Unicode scalar value while retaining byte-based
//! absolute coordinates through each component's UTF-8 width. Dissection returns
//! borrowed string views and line segmentation preserves valid UTF-8 boundaries.

use core::iter::FusedIterator;
use core::num::NonZero;

use aldebaran_span::prelude::{Span, Spanned};

use crate::{
    diff::SourceDiff,
    dissect::SourceDissect,
    iter::SourceIter,
    line::{LineId, LineSegmented, Segmented},
    metadata::{SourceMetadata, Unknown},
    source::Source,
    terminate::{Standard, Terminated},
};

impl<'a> Source<'a> for str {
    type Component = char;

    #[inline]
    fn footprint(&self) -> Option<Span> {
        NonZero::new(self.len()).map(|length| Span::new(0, length))
    }
}

impl<'a> Terminated<'a, Self> for str {
    const DEFAULT_TERMINATOR: Self::TerminatedBy = Self::TerminatedBy::new();

    type TerminatedBy = Standard<char>;
}

impl<'source> SourceDissect<'source> for str {
    type View<'borrow> = &'borrow str;

    #[inline]
    fn dissect<'borrow>(&'borrow self, s0: Span) -> Self::View<'borrow> {
        match self.footprint() {
            Some(s1) => {
                let target_slice = match s1.intersect(s0).as_ref().map(Span::range) {
                    Some(target_range) => &self[target_range],
                    None => &self[s1.end()..],
                };

                target_slice
            }
            _ => self,
        }
    }
}

impl<'source> SourceIter<'source> for str {
    type Iterator<'borrow> = core::str::Chars<'borrow>;

    #[inline]
    fn iter(&self) -> Self::Iterator<'_> {
        self.chars()
    }
}

impl<'a> SourceMetadata<'a> for str {
    type Metadata = Unknown;

    #[inline]
    fn metadata(&self) -> &Self::Metadata {
        &Unknown
    }
}

impl<'source> LineSegmented<'source, str> for &'source str {
    type Segments<'input, I, D>
        = Segmented<'source, 'input, str, I, D>
    where
        I: Iterator<Item = D> + FusedIterator,
        D: Spanned,
        'source: 'input;

    type View = Self;

    #[inline]
    fn segmented<'input, I, D>(line: &'input LineId<'source, str>, data: I) -> Self::Segments<'input, I, D>
    where
        'source: 'input,
        I: Iterator<Item = D> + FusedIterator,
        D: Spanned,
    {
        Segmented::new(line, data)
    }

    #[inline]
    fn window(&self, span: Span) -> Option<Self::View> {
        let target_range = span.range();

        self.get(target_range)
    }
}

impl<'a> SourceDiff<'a> for str {
    #[inline]
    fn same_as(&self, other: &Self) -> bool {
        self == other
    }
}

#[cfg(test)]
mod test {
    use core::{num::NonZero, ops::Deref};

    use aldebaran_span::prelude::Span;

    use crate::line::{LineBreak, LineContent, LineExtent, LineSegmented, Piecewise, SourceLines};
    use crate::source::Source;

    #[test]
    fn terminal_boundary_does_not_segment_source_lines() {
        let source = "abc";
        let terminal_boundary = Span::unit(source.len());

        assert!(source.segment(terminal_boundary).is_none());
    }

    #[test]
    fn source_str_footprint() {
        let source = "Hello, World!\r\n";
        let span = source.footprint();

        assert_eq!(span, Some(Span::new(0, NonZero::new(15).unwrap())));

        let source = "";

        let span = source.footprint();

        assert_eq!(span, None);
    }

    #[test]
    fn line_segment() {
        /// Macro to test the line iterator functions.
        macro_rules! lines {
            () => {};
            (
                $(
                    $target_line:literal => (
                        $(
                            become $target_content:literal
                        )?
                        $(
                            break $target_break:literal
                        )?
                    )
                ),+

                $(,)?
            ) => {
                let target_source = concat!($($target_line),+);

                let target_handle = target_source.extents().expect("non-empty source with empty extents");

                let mut target_lines = target_handle.lines();

                $(
                    let _target_line = target_lines.next().expect("line exists");

                    $(
                        let target_content = _target_line.content().map(LineContent::value).map(Deref::deref);

                        assert_eq!(target_content, Some($target_content));
                    )?

                    $(
                        let target_break = _target_line.content_break().map(LineBreak::line_break).map(Deref::deref);

                        assert_eq!(target_break, Some($target_break));
                    )?
                )+
            };
        }

        lines!(
            "\n" => (break "\n"),
            "Hello, World!\n" => (become "Hello, World!" break "\n"),
            "CRLF, here we go!\r\n" => (become "CRLF, here we go!" break "\r\n"),
            "Single line without break\n" => (become "Single line without break" break "\n"),
            "Hello!\n" => (become "Hello!" break "\n"),
            "Multi-byte character: 🦀\n" => (become "Multi-byte character: 🦀" break "\n"),
            "Emojis: 😂🤣🥲🥹☺️😊😇🙂🙃😉😌😍🥰😘😗😙😚😋😛😝😜🤪🤨🧐🤓😎🥸🤩\r\n" => (become "Emojis: 😂🤣🥲🥹☺️😊😇🙂🙃😉😌😍🥰😘😗😙😚😋😛😝😜🤪🤨🧐🤓😎🥸🤩" break "\r\n"),
            "Tabs\tare\there\n" => (become "Tabs\tare\there" break "\n"),
            "Special characters: !@#$%^&*()\n" => (become "Special characters: !@#$%^&*()" break "\n"),
            "Numbers: 1234567890\n" => (become "Numbers: 1234567890" break "\n"),
            "Mixed: abc123!@#\n" => (become "Mixed: abc123!@#" break "\n"),
            "Unicode: 你好，世界！\n" => (become "Unicode: 你好，世界！" break "\n"),
            "Escape sequences: \t\r\n" => (become "Escape sequences: \t" break "\r\n"),
            "Quotes: \" ' `\n" => (become "Quotes: \" ' `" break "\n"),
            "Brackets: [] {} ()\n" => (become "Brackets: [] {} ()" break "\n"),
        );
    }

    #[test]
    fn source_footprint() {
        macro_rules! footprint {
            (
                $(
                    $target_source:literal => $target_expect:expr
                ),+

                $(,)?
            ) => {
                $(
                    {
                        let source = $target_source;

                        assert_eq!(source.footprint(), $target_expect);
                    }
                )+
            };
        }

        footprint!(
            "1" => Some(Span::new(0, NonZero::new(1).expect("non-zero"))),
            "Hello, World!\r\n" => Some(Span::new(0, NonZero::new(15).expect("non-zero"))),
            "" => None
        );
    }

    // TODO: test segmentation and actually build the reporter

    #[test]
    fn source_segmentation() {
        let source = "Hello, World!\r\n";

        let source_handle = source.extents().expect("non-empty source with empty extents");

        let mut source_lines = source_handle.lines();

        let target_line = source_lines.next().expect("line exists");

        let target_segments: &str = target_line.content().expect("line content exists").value().as_ref();

        assert_eq!(target_segments, "Hello, World!");

        let spans = core::iter::once(Span::new(0, NonZero::<usize>::MIN));

        let mut t = LineSegmented::segmented(&target_line, spans.into_iter());

        assert_eq!(t.next().as_ref().map(Piecewise::<'_, str, Span>::view), Some(&"H"));

        assert_eq!(t.next().as_ref().map(Piecewise::<'_, str, Span>::view), Some(&"ello, World!"));
    }
}
