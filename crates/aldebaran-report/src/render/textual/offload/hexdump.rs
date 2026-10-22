//! Renders raw byte sources as offset-indexed hexadecimal rows with annotation markers.
//!
//! Only ASCII graphic bytes and spaces appear literally in the ASCII column.
//! Other bytes use the configured replacement glyph.

use core::{cmp, fmt, num::NonZero};

use aldebaran_baseline::span::Span;
use aldebaran_print::prelude::{Combine, Print};

use crate::{
    annotated::{Annotated, Annotations},
    prelude::{ErrorKind, Title},
    render::textual::{Offload, OffloadContext},
    report::SourceReport,
};

/// Row width, hexadecimal case, and marker glyphs for a byte dump.
///
/// The default uses 16 bytes per row, uppercase digits, and `.`/`^` markers.
/// Custom glyphs are emitted as supplied, so choose visible single-cell glyphs
/// to preserve row alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// NOTE(invariant): None.
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

impl HexdumpSettings {
    /// Set the row width and use the default case and marker glyphs.
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

    /// Select byte grouping, digit case, and glyphs without validating glyph widths.
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

    /// Source-byte budget per row, independently of the rendered terminal width.
    #[inline]
    #[must_use]
    pub const fn bytes_per_row(&self) -> NonZero<usize> {
        let &Self { bytes_per_row, .. } = self;

        bytes_per_row
    }

    /// Letter case shared by byte values and row offsets.
    #[inline]
    #[must_use]
    pub const fn case(&self) -> HexCase {
        let &Self { case, .. } = self;

        case
    }

    /// ASCII-column substitute for bytes other than graphic ASCII and ordinary space.
    #[inline]
    #[must_use]
    pub const fn replacement(&self) -> char {
        let &Self { replacement, .. } = self;

        replacement
    }

    /// Marker repeated beneath hexadecimal byte values covered by an annotation.
    #[inline]
    #[must_use]
    pub const fn span(&self) -> char {
        let &Self { span, .. } = self;

        span
    }

    /// Insertion marker for a unit annotation at the exclusive source end.
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
// NOTE(invariant): None.
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

    fn print_with_context<W>(&self, writer: &mut W, _: &Self::Context) -> fmt::Result
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

/// Renders byte sources as hexadecimal rows with annotation markers.
///
/// A unit span at the source length marks an insertion after the final byte,
/// including for an empty source.
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

    #[inline]
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
            let row_offset = Hex::from_raw_parts(row_start_offset, 8, hex_case);

            row_offset.sequence(' '.times(2)).print(sink)?;

            for column_index in 0..bytes_per_row {
                match source_row.get(column_index) {
                    Some(source_byte) => {
                        let formatted_byte = Hex::from_raw_parts(*source_byte, 2, hex_case);

                        formatted_byte.sequence(' ').print(sink)?;
                    }
                    None => ' '.times(3).print(sink)?,
                }
            }

            " |".print(sink)?;

            for source_byte in source_row {
                let printable_character = if source_byte.is_ascii_graphic() || *source_byte == b' ' {
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

            if let Some(annotations) = annotations {
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
                        let annotation_start = cmp::max(annotation_span.start(), row_start_offset);
                        let annotation_end = cmp::min(annotation_span.end(), row_end_offset);

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
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests;
