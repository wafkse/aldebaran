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
