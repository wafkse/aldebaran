//! Source capability implementations for borrowed byte slices.
//!
//! Byte slices use one-byte source components, standard line terminators, and
//! unknown location metadata. Dissection preserves borrowed slice views, while
//! line segmentation projects spanned data back onto byte-oriented line windows.

use core::iter::FusedIterator;
use core::num::NonZero;

use crate::diff::SourceDiff;
use crate::dissect::SourceDissect;
use crate::iter::SourceIter;
use crate::line::{LineId, LineSegmented, Segmented};
use crate::metadata::{SourceMetadata, Unknown};
use crate::source::Source;
use crate::terminate::{Standard, Terminated};

use aldebaran_span::prelude::{Span, Spanned};

impl<'a> Source<'a> for [u8] {
    type Component = u8;

    #[inline]
    fn footprint(&self) -> Option<Span> {
        NonZero::new(self.len()).map(|length| Span::new(0, length))
    }
}

impl<'a> Terminated<'a, Self> for [u8] {
    const DEFAULT_TERMINATOR: Self::TerminatedBy = Self::TerminatedBy::new();

    type TerminatedBy = Standard<u8>;
}

impl<'source> SourceIter<'source> for [u8] {
    type Iterator<'borrow> = core::iter::Copied<core::slice::Iter<'borrow, u8>>;

    #[inline]
    fn iter(&self) -> Self::Iterator<'_> {
        self.iter().copied()
    }
}

impl<'a> SourceMetadata<'a> for [u8] {
    type Metadata = Unknown;

    #[inline]
    fn metadata(&self) -> &Self::Metadata {
        &Unknown
    }
}

impl<'source> SourceDissect<'source> for [u8] {
    type View<'borrow> = &'borrow [u8];

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

impl<'source> LineSegmented<'source, [u8]> for &'source [u8] {
    type Segments<'input, I, D>
        = Segmented<'source, 'input, [u8], I, D>
    where
        I: Iterator<Item = D> + FusedIterator,
        D: Spanned,
        'source: 'input;

    type View = Self;

    #[inline]
    fn segmented<'input, I, D>(line: &'input LineId<'source, [u8]>, spans: I) -> Self::Segments<'input, I, D>
    where
        'source: 'input,
        D: Spanned,
        I: Iterator<Item = D> + FusedIterator,
    {
        Segmented::new(line, spans)
    }

    #[inline]
    fn window(&self, span: Span) -> Option<Self::View> {
        let target_range = span.range();

        self.get(target_range)
    }
}

impl<'a> SourceDiff<'a> for [u8] {
    #[inline]
    fn same_as(&self, other: &Self) -> bool {
        self == other
    }
}

#[cfg(test)]
mod test {
    use core::num::NonZero;
    use core::ops::Deref;

    use aldebaran_span::prelude::Span;

    use crate::line::{LineBreak, LineContent, LineExtent, SourceLines};
    use crate::source::Source;

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
                let target_source = concat!($($target_line),+).as_bytes();

                    let target_handle = target_source.extents().expect("non-empty source with empty extents");

                    let mut target_lines = target_handle.lines();

                    $(
                        let _target_line = target_lines.next().expect("line exists");

                        $(
                            let target_content = _target_line.content().map(LineContent::value).map(Deref::deref);

                            assert_eq!(target_content, Some($target_content.as_bytes()));
                        )?

                        $(
                            let target_break = _target_line.content_break().map(LineBreak::line_break).map(Deref::deref);

                            assert_eq!(target_break, Some($target_break.as_bytes()));
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
                        let source = $target_source.as_bytes();

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
}
