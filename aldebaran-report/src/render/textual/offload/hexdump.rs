//! Byte-oriented textual diagnostics rendered as hexadecimal rows.
//!
//! The renderer prints fixed-width byte cells beside an ASCII projection and
//! places annotation markers beneath intersecting rows. Terminal source-boundary
//! diagnostics are rendered as insertion points after the final byte cell.

use core::{fmt, num::NonZero};

use aldebaran_print::prelude::{Combine, Print};
use aldebaran_span::span::Span;

use crate::{
    annotated::{Annotated, Annotations},
    prelude::{ErrorKind, Title},
    render::textual::{Offload, OffloadContext},
    report::SourceReport,
};

/// Presentation settings for byte-oriented diagnostic dumps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HexdumpSettings {
    /// Number of source bytes shown in each rendered row.
    bytes_per_row: NonZero<usize>,

    /// Letter case used for hexadecimal digits.
    case: HexCase,

    /// Glyph used for bytes without a printable ASCII representation.
    replacement: char,

    /// Glyph used to mark bytes covered by an annotation span.
    span: char,

    /// Glyph used to mark an annotation at the source boundary.
    point: char,
}

// NOTE(invariant): Every rendered row has a strictly positive byte capacity.
impl HexdumpSettings {
    /// Construct hexdump settings with an explicit row width.
    #[inline]
    #[must_use]
    pub const fn new(bytes_per_row: NonZero<usize>) -> Self {
        let case = HexCase::Upper;
        let replacement = '.';
        let span = '^';
        let point = '^';

        Self {
            bytes_per_row,
            case,
            replacement,
            span,
            point,
        }
    }

    /// Construct hexdump settings from their rendering parts.
    #[inline]
    #[must_use]
    pub const fn from_raw_parts(bytes_per_row: NonZero<usize>, case: HexCase, replacement: char, span: char, point: char) -> Self {
        Self {
            bytes_per_row,
            case,
            replacement,
            span,
            point,
        }
    }

    /// Retrieve the number of bytes rendered per row.
    #[inline]
    #[must_use]
    pub const fn bytes_per_row(&self) -> NonZero<usize> {
        let &Self { bytes_per_row, .. } = self;

        bytes_per_row
    }

    /// Retrieve the hexadecimal letter case.
    #[inline]
    #[must_use]
    pub const fn case(&self) -> HexCase {
        let &Self { case, .. } = self;

        case
    }

    /// Retrieve the non-graphic byte replacement glyph.
    #[inline]
    #[must_use]
    pub const fn replacement(&self) -> char {
        let &Self { replacement, .. } = self;

        replacement
    }

    /// Retrieve the annotation span glyph.
    #[inline]
    #[must_use]
    pub const fn span(&self) -> char {
        let &Self { span, .. } = self;

        span
    }

    /// Retrieve the source-boundary point glyph.
    #[inline]
    #[must_use]
    pub const fn point(&self) -> char {
        let &Self { point, .. } = self;

        point
    }
}

impl Default for HexdumpSettings {
    #[inline]
    fn default() -> Self {
        let bytes_per_row = NonZero::<usize>::MIN.saturating_add(15);

        Self::new(bytes_per_row)
    }
}

/// Letter case used for hexadecimal digits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HexCase {
    /// Uppercase hexadecimal digits.
    Upper,

    /// Lowercase hexadecimal digits.
    Lower,
}

impl Default for HexCase {
    #[inline]
    fn default() -> Self {
        Self::Upper
    }
}

/// Hexadecimal value with an explicit field width and letter case.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Hex<T> {
    /// Value rendered in hexadecimal notation.
    value: T,

    /// Minimum number of hexadecimal digits emitted for the value.
    width: usize,

    /// Letter case used for hexadecimal alphabetic digits.
    case: HexCase,
}

impl<T> Hex<T> {
    /// Construct an uppercase hexadecimal value without minimum padding.
    const fn new(value: T) -> Self {
        let width = 0;
        let case = HexCase::Upper;

        Self { value, width, case }
    }

    /// Construct a hexadecimal value from its formatting parts.
    const fn from_raw_parts(value: T, width: usize, case: HexCase) -> Self {
        Self { value, width, case }
    }

    /// Construct a hexadecimal value padded to at least two digits.
    const fn two(value: T, case: HexCase) -> Self {
        Self::from_raw_parts(value, 2, case)
    }

    /// Construct a hexadecimal value padded to at least eight digits.
    const fn eight(value: T, case: HexCase) -> Self {
        Self::from_raw_parts(value, 8, case)
    }
}

impl<T> Default for Hex<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T> Print for Hex<T>
where
    T: fmt::LowerHex + fmt::UpperHex,
{
    type Context = ();

    fn print_with_ctx<W>(&self, writer: &mut W, _: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        let Self { value, width, case } = self;

        match case {
            HexCase::Upper => write!(writer, "{value:0width$X}"),
            HexCase::Lower => write!(writer, "{value:0width$x}"),
        }
    }
}

/// Textual offload renderer for byte-oriented diagnostic sources.
///
/// Each source row is rendered as hexadecimal byte cells with an ASCII
/// projection. Annotation spans are projected beneath intersecting rows, while a
/// terminal unit span is rendered as an insertion marker after the final cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Hexdump {}

impl<'source, E> Offload<'source, E> for Hexdump
where
    E: SourceReport<'source, Source = [u8]>,
    <<E::Kind as ErrorKind>::Descriptor as Print>::Context: Default,
    <E::Title as Title>::Context: Default,
    <<<E::Annotations as Annotations>::Annotation as Annotated>::Title as Title>::Context: Default,
{
    type Input<'input>
        = HexdumpSettings
    where
        'source: 'input;

    type Output = ();
    type Error = fmt::Error;

    fn offload<'input, W>(error: &'source E, ctx: &mut OffloadContext<'source, 'input, W, Self, E>) -> Result<Self::Output, Self::Error>
    where
        'source: 'input,
        W: fmt::Write,
    {
        let settings = ctx.present().input();
        let bytes_per_row = settings.bytes_per_row().get();
        let hex_case = settings.case();
        let replacement_glyph = settings.replacement();
        let span_glyph = settings.span();
        let point_glyph = settings.point();
        let sink = ctx.sink();
        let source = error.source();
        let source_length = source.len();

        error.kind().descriptor().print(sink)?;
        ':'.sequence(' ').sequence(error.title().content()).sequence('\n').print(sink)?;

        let annotations = error.annotations().list();

        let source_rows = source.is_empty().then_some(source).into_iter().chain(source.chunks(bytes_per_row));

        for (row_index, source_row) in source_rows.enumerate() {
            let row_start_offset = row_index.saturating_mul(bytes_per_row);
            let row_end_offset = row_start_offset.saturating_add(source_row.len());
            let row_offset = Hex::eight(row_start_offset, hex_case);

            row_offset.sequence(' '.times(2)).print(sink)?;

            for column_index in 0..bytes_per_row {
                match source_row.get(column_index) {
                    Some(source_byte) => {
                        let formatted_byte = Hex::two(*source_byte, hex_case);

                        formatted_byte.sequence(' ').print(sink)?;
                    }
                    None => ' '.times(3).print(sink)?,
                }
            }

            " |".print(sink)?;

            for source_byte in source_row {
                let printable_character = if source_byte.is_ascii_graphic() || source_byte.is_ascii_whitespace() {
                    char::from(*source_byte)
                } else {
                    replacement_glyph
                };

                printable_character.print(sink)?;
            }

            for _ in source_row.len()..bytes_per_row {
                ' '.print(sink)?;
            }

            "|\n".print(sink)?;

            match annotations {
                Some(annotations) => {
                    for annotation in annotations.iter() {
                        let annotation_span = annotation.target();
                        let is_source_boundary = annotation_span == Span::unit(source_length);
                        let is_final_row = row_end_offset == source_length;

                        if is_source_boundary && is_final_row {
                            ' '.times(10).print(sink)?;

                            for _ in 0..source_length - row_start_offset {
                                ' '.times(3).print(sink)?;
                            }

                            point_glyph
                                .sequence(' ')
                                .sequence(annotation.message().content())
                                .sequence('\n')
                                .print(sink)?;
                        } else {
                            let annotation_start = core::cmp::max(annotation_span.start(), row_start_offset);
                            let annotation_end = core::cmp::min(annotation_span.end(), row_end_offset);

                            if annotation_start < annotation_end {
                                ' '.times(10).print(sink)?;

                                for _ in row_start_offset..annotation_start {
                                    ' '.times(3).print(sink)?;
                                }

                                for _ in annotation_start..annotation_end {
                                    span_glyph.times(2).sequence(' ').print(sink)?;
                                }

                                annotation.message().content().sequence('\n').print(sink)?;
                            }
                        }
                    }
                }
                None => {}
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use core::num::NonZero;

    use aldebaran_span::span::Span;

    use crate::{
        annotated::{InlineAnnotations, label::Label},
        render::{
            RenderMut,
            textual::{Present, Textual},
        },
        report::{Report, kind::Simple},
    };

    use super::{HexCase, Hexdump, HexdumpSettings};

    #[derive(Debug)]
    struct Diagnostic(InlineAnnotations<Label<&'static str>, 1>);

    impl Report for Diagnostic {
        type Title = str;
        type Kind = Simple;
        type Annotations = InlineAnnotations<Label<&'static str>, 1>;

        #[inline]
        fn kind(&self) -> &Self::Kind {
            &Simple::Error
        }

        #[inline]
        fn title(&self) -> &Self::Title {
            "malformed bytes"
        }

        #[inline]
        fn annotations(&self) -> &Self::Annotations {
            let Self(annotations) = self;

            annotations
        }
    }

    #[test]
    fn renders_byte_spans() {
        let source = [0xde, 0xad, 0xbe, 0xef, 0x00, b'A'];
        let length = NonZero::<usize>::MIN.saturating_add(1);
        let primary = Label::new("invalid bytes", Span::new(1, length));
        let related = [Label::new("insert here", Span::unit(source.len()))];
        let report = Diagnostic(InlineAnnotations::new(primary, related));
        let attached = report.attach(source.as_slice());
        let mut output = String::new();
        let mut renderer = Textual::<'_, _, Hexdump, _>::new(&mut output);
        let settings = Present::from_input(HexdumpSettings::default());

        renderer
            .render_mut_with_input(settings, &attached)
            .expect("hexdump rendering should succeed");

        assert!(output.contains("error: malformed bytes"));
        assert!(output.contains("00000000  DE AD BE EF 00 41"));
        assert!(output.contains("|.....A"));
        assert!(output.contains("^^ ^^ invalid bytes"));
        assert!(output.contains("^ insert here"));
    }

    #[test]
    fn customizes_hexdump_presentation() {
        let source = [0xab, b'A', 0x00];
        let length = NonZero::<usize>::MIN.saturating_add(1);
        let primary = Label::new("invalid bytes", Span::new(0, length));
        let related = [Label::new("insert here", Span::unit(source.len()))];
        let report = Diagnostic(InlineAnnotations::new(primary, related));
        let attached = report.attach(source.as_slice());
        let mut output = String::new();
        let mut renderer = Textual::<'_, _, Hexdump, _>::new(&mut output);
        let bytes_per_row = NonZero::<usize>::MIN.saturating_add(2);
        let settings = HexdumpSettings::from_raw_parts(bytes_per_row, HexCase::Lower, '?', '~', '!');
        let settings = Present::from_input(settings);

        renderer
            .render_mut_with_input(settings, &attached)
            .expect("custom hexdump rendering should succeed");

        assert!(output.contains("00000000  ab 41 00"));
        assert!(output.contains("|?A?|"));
        assert!(output.contains("~~ ~~ invalid bytes"));
        assert!(output.contains("! insert here"));
    }
}
