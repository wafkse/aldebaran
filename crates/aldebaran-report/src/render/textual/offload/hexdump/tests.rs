use core::num::NonZero;

use aldebaran_baseline::span::Span;

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
// NOTE(invariant): None.
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
    let settings = Present::with_input(HexdumpSettings::default());

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
    let settings = HexdumpSettings::raw(bytes_per_row, HexCase::Lower, '?', '~', '!');
    let settings = Present::with_input(settings);

    renderer
        .render_mut_with_input(settings, &attached)
        .expect("custom hexdump rendering should succeed");

    assert!(output.contains("00000000  ab 41 00"));
    assert!(output.contains("|?A?|"));
    assert!(output.contains("~~ ~~ invalid bytes"));
    assert!(output.contains("! insert here"));
}

#[test]
fn ascii_projection_replaces_controls_and_preserves_space() {
    let source = [b'A', b'\t', b'\n', b'\r', 0x0b, 0x0c, 0x1b, 0x7f, b' ', b'~', 0xff];
    let report = Diagnostic(InlineAnnotations::new(
        Label::new("bytes", Span::unit(0)),
        [Label::new("end", Span::unit(source.len()))],
    ));
    let attached = report.attach(source.as_slice());
    let mut output = String::new();
    let mut renderer = Textual::<'_, _, Hexdump, _>::new(&mut output);
    let settings = HexdumpSettings::raw(NonZero::new(source.len()).expect("nonempty source"), HexCase::Upper, '?', '^', '^');

    renderer
        .render_mut_with_input(Present::with_input(settings), &attached)
        .expect("render controls");

    assert!(output.contains("|A??????? ~?|"), "incorrect ASCII projection {output:?}");
    assert!(
        !output.contains(['\t', '\r', '\x0b', '\x0c', '\x1b', '\x7f']),
        "control byte escaped into output {output:?}"
    );
}
