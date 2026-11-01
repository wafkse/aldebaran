//! Printable parts that borrow the report data and layout resolved by Fancy.

use core::{
    fmt,
    num::NonZero,
    ops::{Range, RangeInclusive},
};

use aldebaran_ansi::{brush::Paintable, predicate::Should, prelude::Style};
use aldebaran_print::prelude::{Combine, Print};

use crate::{
    annotated::{Annotated, title::Title},
    prelude::ErrorKind,
};

use super::super::run::GlyphRun;
use super::super::{Colored, charset::shape::Delimit, theme::HeaderTheme};
use super::Gutter;

/// Prints the report classification and title without adding a trailing newline.
// NOTE(invariant): None.
pub(in crate::render::textual::offload::fancy::detail) struct Header<'a, K, T: ?Sized> {
    /// Report classification supplying its printable descriptor.
    pub(in crate::render::textual::offload::fancy::detail) kind: &'a K,

    /// Report title supplying its printable content.
    pub(in crate::render::textual::offload::fancy::detail) title: &'a T,

    /// Heading styles already selected from report metadata.
    pub(in crate::render::textual::offload::fancy::detail) theme: &'a HeaderTheme,

    /// Whether to emit ANSI presentation.
    pub(in crate::render::textual::offload::fancy::detail) colored: &'a Colored,
}

impl<K: ErrorKind, T: Title + ?Sized> Print for Header<'_, K, T>
where
    <K::Descriptor as Print>::Context: Default,
    T::Context: Default,
{
    type Context = ();

    fn print_with_context<W>(&self, writer: &mut W, _: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        let Self {
            kind,
            title,
            theme,
            colored,
        } = self;

        let colored = Should::tuple((*colored, Colored::colored));

        kind.descriptor()
            .stylable()
            .styled(theme.kind())
            .only_when(&colored)
            .sequence(": ")
            .sequence(title.content().stylable().styled(theme.title()).only_when(&colored))
            .print(writer)
    }
}

/// Prints the line-number column and its padding, leaving the separator to the caller.
// NOTE(invariant): LineNumber.IsSome => LineNumber.Digits <= Gutter.LineNumberDigitWidth
pub(in crate::render::textual::offload::fancy::detail) struct GutterLine<'a> {
    /// Prepared digit width and surrounding spaces.
    pub(in crate::render::textual::offload::fancy::detail) gutter: &'a Gutter,

    /// Absent for continuation, marker, and message rows.
    pub(in crate::render::textual::offload::fancy::detail) line_number: Option<NonZero<usize>>,

    /// Resolved line-number style.
    pub(in crate::render::textual::offload::fancy::detail) style: Style,

    /// Whether to emit ANSI presentation.
    pub(in crate::render::textual::offload::fancy::detail) colored: &'a Colored,
}

impl Print for GutterLine<'_> {
    type Context = ();

    fn print_with_context<W>(&self, writer: &mut W, _: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        let &Self {
            gutter,
            line_number,
            style,
            colored,
        } = self;

        let &Gutter {
            line_number_digit_width,
            line_column_padding,
            ..
        } = gutter;

        let colored = Should::tuple((colored, Colored::colored));

        let count = match line_number {
            Some(line_number) => line_column_padding.left() + line_number_digit_width - (line_number.get().ilog10() + 1),
            None => gutter.width(),
        };
        let left = GlyphRun { glyph: ' ', count };
        let count = line_column_padding.right();
        let right = GlyphRun { glyph: ' ', count };

        match line_number {
            Some(line_number) => left
                .sequence(line_number.stylable().styled(style).only_when(&colored))
                .sequence(right)
                .print(writer),
            None => left.print(writer),
        }
    }
}

/// Prints the bracketed source origin and selected original line numbers.
// NOTE(invariant): None.
pub(in crate::render::textual::offload::fancy::detail) struct SourceLocation<'a, M: ?Sized> {
    /// Source-specific printable origin.
    pub(in crate::render::textual::offload::fancy::detail) metadata: &'a M,

    /// Inclusive physical line-number range.
    pub(in crate::render::textual::offload::fancy::detail) line_number_range: &'a RangeInclusive<NonZero<usize>>,

    /// Resolved line-number style.
    pub(in crate::render::textual::offload::fancy::detail) style: Style,

    /// Whether to emit ANSI presentation.
    pub(in crate::render::textual::offload::fancy::detail) colored: &'a Colored,
}

impl<M: Print + ?Sized> Print for SourceLocation<'_, M>
where
    M::Context: Default,
{
    type Context = ();

    fn print_with_context<W>(&self, writer: &mut W, _: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        let &Self {
            metadata,
            line_number_range,
            style,
            colored,
        } = self;

        let colored = Should::tuple((colored, Colored::colored));

        '['.sequence(metadata)
            .sequence(':')
            .sequence(line_number_range.start().stylable().styled(style).only_when(&colored))
            .sequence(':')
            .sequence(line_number_range.end().stylable().styled(style).only_when(&colored))
            .sequence(']')
            .print(writer)
    }
}

/// Prints the excerpt's opening or closing frame without adding a newline.
pub(in crate::render::textual::offload::fancy::detail) enum ExcerptBoundary<'a, M: ?Sized> {
    /// Gutter padding, opening corner and stroke, then the source location.
    Start {
        /// Alignment shared with the excerpt rows.
        gutter: &'a Gutter,

        /// Existing frame glyph settings.
        delimit: Delimit,

        /// Borrowed source origin and selected line range.
        location: SourceLocation<'a, M>,
    },

    /// Bottom strokes across the gutter, followed by the closing corner.
    End {
        /// Alignment shared with the excerpt rows.
        gutter: &'a Gutter,

        /// Existing frame glyph settings.
        delimit: Delimit,
    },
}

impl<M: Print + ?Sized> Print for ExcerptBoundary<'_, M>
where
    M::Context: Default,
{
    type Context = ();

    fn print_with_context<W>(&self, writer: &mut W, _: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        match self {
            Self::Start { gutter, delimit, location } => {
                let count = gutter.width();

                GlyphRun { glyph: ' ', count }
                    .sequence(delimit.corner().top_left())
                    .sequence(delimit.top())
                    .sequence(location)
                    .print(writer)
            }
            Self::End { gutter, delimit } => {
                let glyph = delimit.bottom();
                let count = gutter.width();

                GlyphRun { glyph, count }.sequence(delimit.corner().bottom_right()).print(writer)
            }
        }
    }
}

/// Prints source text after layout has normalized and wrapped it.
// NOTE(invariant): Text = PreparedLayout.Output
pub(in crate::render::textual::offload::fancy::detail) struct SourcePart<'a>(
    pub(in crate::render::textual::offload::fancy::detail) &'a str,
);

impl Print for SourcePart<'_> {
    type Context = ();

    #[inline]
    fn print_with_context<W>(&self, writer: &mut W, _: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        let Self(text) = self;

        text.print(writer)
    }
}

/// Prints an annotation marker in its assigned style across the selected columns.
// NOTE(invariant): None.
pub(in crate::render::textual::offload::fancy::detail) struct Underline {
    /// Assigned annotation glyph.
    pub(in crate::render::textual::offload::fancy::detail) marker: char,

    /// Resolved annotation style.
    pub(in crate::render::textual::offload::fancy::detail) style: Style,

    /// Number of displayed marker glyphs.
    pub(in crate::render::textual::offload::fancy::detail) width: u32,
}

impl Print for Underline {
    type Context = Colored;

    #[inline]
    fn print_with_context<W>(&self, writer: &mut W, colored: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        let &Self { marker, style, width } = self;

        let colored = Should::tuple((colored, Colored::colored));

        GlyphRun {
            glyph: marker,
            count: width,
        }
        .stylable()
        .styled(style)
        .only_when(&colored)
        .print(writer)
    }
}

/// Marks a terminal-column range with an annotation's underline glyph.
// NOTE(invariant): ColumnRange.Start < ColumnRange.End
pub(in crate::render::textual::offload::fancy::detail) struct UnderlineSegment {
    /// Terminal columns occupied by this segment.
    pub(in crate::render::textual::offload::fancy::detail) column_range: Range<usize>,

    /// Prepared marker run.
    pub(in crate::render::textual::offload::fancy::detail) underline: Underline,
}

/// Prints one marker lane, leaving the gutter and newline to the caller.
// NOTE(invariant): OrderedNonoverlapping(UnderlineSegmentList.ColumnRange)
pub(in crate::render::textual::offload::fancy::detail) struct UnderlineRow<'a> {
    /// Ordered segments selected for this lane.
    pub(in crate::render::textual::offload::fancy::detail) underline_segment_list: &'a [UnderlineSegment],

    /// Whether to emit ANSI presentation.
    pub(in crate::render::textual::offload::fancy::detail) colored: &'a Colored,
}

impl Print for UnderlineRow<'_> {
    type Context = ();

    fn print_with_context<W>(&self, writer: &mut W, _: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        let Self {
            underline_segment_list,
            colored,
        } = self;

        let mut rightmost = usize::MIN;

        for UnderlineSegment { column_range, underline } in *underline_segment_list {
            let padding = u32::try_from(column_range.start.saturating_sub(rightmost)).unwrap_or(u32::MAX);

            GlyphRun {
                glyph: ' ',
                count: padding,
            }
            .print(writer)?;
            underline.print_with_context(writer, colored)?;
            rightmost = column_range.end;
        }

        Ok(())
    }
}

/// Prints an annotation marker in brackets before the message separator.
// NOTE(invariant): None.
pub(in crate::render::textual::offload::fancy::detail) struct AnnotationReference<'a> {
    /// Assigned annotation glyph.
    pub(in crate::render::textual::offload::fancy::detail) marker: char,

    /// Resolved annotation style.
    pub(in crate::render::textual::offload::fancy::detail) style: Style,

    /// Whether to emit ANSI presentation.
    pub(in crate::render::textual::offload::fancy::detail) colored: &'a Colored,

    /// Whether colored output emphasizes the brackets.
    pub(in crate::render::textual::offload::fancy::detail) emphasized: bool,
}

impl Print for AnnotationReference<'_> {
    type Context = ();

    fn print_with_context<W>(&self, writer: &mut W, _: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        let &Self {
            marker,
            style,
            colored,
            emphasized,
        } = self;

        let bracket_color = || colored.colored() && emphasized;
        let colored = Should::tuple((colored, Colored::colored));

        '['.stylable()
            .bold()
            .only_when(&bracket_color)
            .sequence(marker.stylable().styled(style).only_when(&colored))
            .sequence(']'.stylable().bold().only_when(&bracket_color))
            .print(writer)
    }
}

/// Prints an annotation marker and its message. The caller adds indentation and
/// a newline.
// NOTE(invariant): Reference = AssignedPresentation(Annotation.Target)
pub(in crate::render::textual::offload::fancy::detail) struct AnnotationMessage<'a, A> {
    /// Borrowed annotation supplying the message content.
    pub(in crate::render::textual::offload::fancy::detail) annotation: &'a A,

    /// Marker presentation resolved before printing.
    pub(in crate::render::textual::offload::fancy::detail) reference: AnnotationReference<'a>,
}

impl<A: Annotated> Print for AnnotationMessage<'_, A>
where
    <A::Title as Title>::Context: Default,
{
    type Context = ();

    fn print_with_context<W>(&self, writer: &mut W, _: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        let Self { annotation, reference } = self;

        reference.sequence(": ").sequence(annotation.message().content()).print(writer)
    }
}

#[cfg(test)]
mod tests {
    //! Tests module for the [`part`] module.
    //!
    //! [`part`]: super

    //! Output boundaries, repeatability, and failure behavior of borrowed parts.

    use core::{fmt, num::NonZero};

    use aldebaran_baseline::prelude::Span;
    use aldebaran_print::prelude::{Combine, Print};

    use crate::{
        prelude::Label,
        render::textual::offload::fancy::{
            Colored, FancySettings,
            charset::Charset,
            layout::{SourceSpacing, padding::HorizontalPadding},
        },
    };

    use super::{
        AnnotationMessage, AnnotationReference, ExcerptBoundary, Gutter, SourceLocation, SourcePart, Style, Underline, UnderlineRow,
        UnderlineSegment,
    };

    /// Reject output once the byte budget is exhausted, without retaining text.
    // NOTE(invariant): None.
    struct Limited {
        /// Remaining writable bytes before the next sink failure.
        remaining: usize,
    }

    impl fmt::Write for Limited {
        fn write_str(&mut self, text: &str) -> fmt::Result {
            let &mut Self { ref mut remaining } = self;

            *remaining = remaining.checked_sub(text.len()).ok_or(fmt::Error)?;

            Ok(())
        }
    }

    #[test]
    fn gutter_and_boundaries_leave_separators_and_newlines_to_composition() {
        let gutter = Gutter {
            line_number_digit_width: 2,
            line_column_padding: HorizontalPadding::tuple((1, 2)),
            source_spacing: SourceSpacing::Tabs {
                width: NonZero::new(4).expect("tab width"),
                amount: NonZero::new(2).expect("tab count"),
            },
        };
        let line_number_range = NonZero::new(9).expect("first line")..=NonZero::new(12).expect("last line");
        let fancy_settings = FancySettings::standard();
        let style = Style::empty();
        let colored = Colored::No;
        let line_number = NonZero::new(9);
        let line = gutter.line(line_number, style, &colored);
        let mut output = String::new();

        line.print(&mut output).expect("print digit column");

        assert_eq!(output, "  9  ", "gutter adapter must end before the separator");

        output.clear();

        gutter
            .prefix(line_number, &fancy_settings)
            .sequence(SourcePart("中e\u{301}"))
            .sequence('\n')
            .print(&mut output)
            .expect("compose source row");

        assert_eq!(output, "  9  |        中e\u{301}\n", "source text and spacing must print verbatim");

        for (charset, expected) in [
            (Charset::ASCII, "     +-[file.v:9:12]\n-----+"),
            (Charset::UNICODE, "     ┌─[file.v:9:12]\n─────┘"),
        ] {
            let start = ExcerptBoundary::Start {
                gutter: &gutter,
                delimit: charset.delimit(),
                location: SourceLocation {
                    metadata: "file.v",
                    line_number_range: &line_number_range,
                    style,
                    colored: &colored,
                },
            };
            let end = ExcerptBoundary::<()>::End {
                gutter: &gutter,
                delimit: charset.delimit(),
            };

            output.clear();

            start.sequence('\n').sequence(end).print(&mut output).expect("compose boundaries");

            assert_eq!(output, expected, "only composition should supply the boundary newline");
        }
    }

    #[test]
    fn annotation_parts_can_be_reprinted_after_any_sink_failure() {
        let underline_segment_list = [
            UnderlineSegment {
                column_range: 1..3,
                underline: Underline {
                    marker: '^',
                    style: Style::empty(),
                    width: 2,
                },
            },
            UnderlineSegment {
                column_range: 5..6,
                underline: Underline {
                    marker: '*',
                    style: Style::empty(),
                    width: 1,
                },
            },
        ];
        let annotation = Label::new("detail", Span::unit(usize::MIN));

        for (colored, emphasized, expected) in [
            (Colored::No, true, " ^^  *\n[^]: detail"),
            (Colored::Yes, false, " ^^\x1b[0m  *\x1b[0m\n[^\x1b[0m]: detail"),
            (
                Colored::Yes,
                true,
                " ^^\x1b[0m  *\x1b[0m\n\x1b[1m[\x1b[0m^\x1b[0m\x1b[1m]\x1b[0m: detail",
            ),
        ] {
            let row = UnderlineRow {
                underline_segment_list: &underline_segment_list,
                colored: &colored,
            };
            let message = AnnotationMessage {
                annotation: &annotation,
                reference: AnnotationReference {
                    marker: '^',
                    style: Style::empty(),
                    colored: &colored,
                    emphasized,
                },
            };
            let composed = row.sequence('\n').sequence(message);

            for remaining in usize::MIN..expected.len() {
                let mut sink = Limited { remaining };
                let result = composed.print(&mut sink);
                let mut output = String::new();

                assert!(result.is_err(), "insufficient sink budget must fail at {remaining}");

                composed.print(&mut output).expect("reprint after sink failure");

                assert_eq!(output, expected, "printing must not consume prepared annotation state");
            }
        }
    }
}
