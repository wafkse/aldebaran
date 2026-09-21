//! Byte-oriented textual diagnostics rendered as hexadecimal rows.
//!
//! The renderer prints fixed-width byte cells beside an ASCII projection and
//! places annotation markers beneath intersecting rows. Terminal source-boundary
//! diagnostics are rendered as insertion points after the final byte cell.

use core::{fmt, num::NonZero};

use aldebaran_print::prelude::{Combine, Print};

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
}

// NOTE(invariant): Every rendered row has a strictly positive byte capacity.
impl HexdumpSettings {
    /// Construct hexdump settings with an explicit row width.
    #[inline]
    #[must_use]
    pub const fn new(bytes_per_row: NonZero<usize>) -> Self {
        Self { bytes_per_row }
    }

    /// Retrieve the number of bytes rendered per row.
    #[inline]
    #[must_use]
    pub const fn bytes_per_row(&self) -> NonZero<usize> {
        let &Self { bytes_per_row } = self;

        bytes_per_row
    }
}

impl Default for HexdumpSettings {
    #[inline]
    fn default() -> Self {
        let bytes_per_row = NonZero::<usize>::MIN.saturating_add(15);

        Self { bytes_per_row }
    }
}

/// Textual offload renderer for byte-oriented diagnostic sources.
///
/// Each source row is rendered as hexadecimal byte cells with an ASCII
/// projection. Annotation spans are projected beneath intersecting rows, while a
/// terminal unit span is rendered as an insertion marker after the final cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Hexdump {}

impl Hexdump {
    /// Render one byte row and its ASCII projection.
    fn row<W>(sink: &mut W, row_start: usize, row: &[u8], width: usize) -> fmt::Result
    where
        W: fmt::Write,
    {
        write!(sink, "{row_start:08X}  ")?;

        for column in 0..width {
            match row.get(column) {
                Some(byte) => write!(sink, "{byte:02X} ")?,
                None => sink.write_str("   ")?,
            }
        }

        sink.write_str(" |")?;

        for byte in row {
            let printable = match byte {
                0x20..=0x7e => char::from(*byte),
                _ => '.',
            };

            sink.write_char(printable)?;
        }

        for _ in row.len()..width {
            sink.write_char(' ')?;
        }

        sink.write_str("|\n")
    }

    /// Render one annotation when its source span intersects this row.
    fn annotation<W, A>(sink: &mut W, annotation: &A, row_start: usize, row_len: usize, source_len: usize) -> fmt::Result
    where
        W: fmt::Write,
        A: Annotated + ?Sized,
        <A::Title as Title>::Context: Default,
    {
        let row_end = row_start.saturating_add(row_len);
        let span = annotation.target();
        let boundary = span.start() == source_len && span.length() == NonZero::<usize>::MIN;
        let final_row = row_end == source_len;

        match (boundary, final_row) {
            (true, true) => Self::point_marker(sink, annotation, source_len - row_start),
            _ => {
                let start = core::cmp::max(span.start(), row_start);
                let end = core::cmp::min(span.end(), row_end);

                if start < end {
                    Self::span_marker(sink, annotation, start - row_start, end - start)
                } else {
                    Ok(())
                }
            }
        }
    }

    /// Render a whole-byte source span beneath one hexdump row.
    fn span_marker<W, A>(sink: &mut W, annotation: &A, offset: usize, length: usize) -> fmt::Result
    where
        W: fmt::Write,
        A: Annotated + ?Sized,
        <A::Title as Title>::Context: Default,
    {
        sink.write_str("          ")?;

        for _ in 0..offset {
            sink.write_str("   ")?;
        }

        for _ in 0..length {
            sink.write_str("^^ ")?;
        }

        annotation.message().content().sequence('\n').print(sink)
    }

    /// Render a source-boundary marker between byte cells.
    fn point_marker<W, A>(sink: &mut W, annotation: &A, offset: usize) -> fmt::Result
    where
        W: fmt::Write,
        A: Annotated + ?Sized,
        <A::Title as Title>::Context: Default,
    {
        sink.write_str("          ")?;

        for _ in 0..offset {
            sink.write_str("   ")?;
        }

        '^'.sequence(' ')
            .sequence(annotation.message().content())
            .sequence('\n')
            .print(sink)
    }
}

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
        let width = ctx.present().input().bytes_per_row().get();
        let sink = ctx.sink();
        let source = error.source();

        error.kind().descriptor().print(sink)?;
        ':'.sequence(' ').sequence(error.title().content()).sequence('\n').print(sink)?;

        let annotations = error.annotations().list();

        if source.is_empty() {
            Self::row(sink, 0, source, width)?;

            match annotations {
                Some(annotations) => {
                    for annotation in annotations.iter() {
                        Self::annotation(sink, annotation, 0, 0, 0)?;
                    }
                }
                None => {}
            }
        } else {
            for (row_index, row) in source.chunks(width).enumerate() {
                let row_start = row_index.saturating_mul(width);

                Self::row(sink, row_start, row, width)?;

                match annotations {
                    Some(annotations) => {
                        for annotation in annotations.iter() {
                            Self::annotation(sink, annotation, row_start, row.len(), source.len())?;
                        }
                    }
                    None => {}
                }
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

    use super::{Hexdump, HexdumpSettings};

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
}
