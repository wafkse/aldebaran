//! Borrowed output parts, using geometry and presentation resolved by Fancy.

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

/// The classification and title, without a trailing newline.
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

/// The digit column and its padding, excluding the vertical separator.
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

/// The bracketed source origin and selected original line numbers.
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

/// Opening and closing parts of the excerpt frame, without newlines.
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

/// Visible source text already normalized and wrapped by layout.
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

/// A marker run with its identity and style resolved before printing.
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

/// An underline at a position assigned during lane preparation.
// NOTE(invariant): ColumnRange.Start < ColumnRange.End
pub(in crate::render::textual::offload::fancy::detail) struct UnderlineSegment {
    /// Terminal columns occupied by this segment.
    pub(in crate::render::textual::offload::fancy::detail) column_range: Range<usize>,

    /// Prepared marker run.
    pub(in crate::render::textual::offload::fancy::detail) underline: Underline,
}

/// One prepared marker lane, excluding the gutter and newline.
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

/// A bracketed annotation marker, without the message separator.
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

/// An annotation reference and message, without indentation or a newline.
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
mod tests;
