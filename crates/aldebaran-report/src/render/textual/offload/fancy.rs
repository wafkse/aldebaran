//! Styled diagnostics with annotated source excerpts.

pub mod charset;
pub mod component;
pub mod layout;
pub mod setting;
pub mod theme;

pub use self::{
    component::{ByLines, Header},
    layout::LayoutSettings,
    setting::{Colored, FancySettings, Fancyness},
    theme::{DefaultTheme, Theme},
};

use core::{borrow::BorrowMut, fmt, marker};

use aldebaran_print::visualize::Visualize;

use aldebaran_print::prelude::Print;

use aldebaran_baseline::span::Span;

use crate::{
    annotated::{Annotated, Annotations},
    prelude::{ErrorKind, Title},
    report::SourceReport,
};
use aldebaran_source::prelude::{LineSegmented, Location, Source, SourceLines, SourceMetadata};

use crate::render::textual::{Offload, OffloadContext};

/// Renders a report heading, selected source lines, and annotation markers.
///
/// Annotation spans retain source coordinates. For `str`, spans count UTF-8
/// bytes, while layout uses terminal-cell widths. Tabs advance to four-column
/// stops and other control characters appear as replacement characters.
///
/// Targets are clipped to visible source content. Empty sources and targets
/// outside the source content produce only the heading. A [`Span::unit`] at EOF
/// marks an insertion after nonempty unterminated content. This marker appears
/// on the next display row when the source ends with a line terminator. Longer
/// spans beginning at EOF do not mark an insertion.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
// NOTE(invariant): None.
pub struct Fancy<T = DefaultTheme>(marker::PhantomData<T>)
where
    T: Theme;

impl<'source, E, T> Offload<'source, E> for Fancy<T>
where
    E: SourceReport<'source>,
    T: Theme + 'static,
    E::Source: SourceLines<'source> + SourceMetadata<'source>,
    <E::Source as SourceMetadata<'source>>::Metadata: Location,
    <E::Source as SourceLines<'source>>::Line: Print,
    <<E::Source as SourceMetadata<'source>>::Metadata as Print>::Context: Default,
    <<E::Source as SourceLines<'source>>::Line as LineSegmented<'source, E::Source>>::View: Visualize,
    //
    <<E::Kind as ErrorKind>::Descriptor as Print>::Context: Default,
    <E::Title as Title>::Context: Default,
    <<<E::Annotations as Annotations>::Annotation as Annotated>::Title as Title>::Context: Default,
{
    type Input<'input>
        = FancySettings<T>
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
        Header::offload(error, ctx.referenced().borrow_mut())?;

        let source = error.source();
        let segment = error.target().and_then(|target| {
            let source_size = source.size();

            // Select real context without changing the annotation's coordinates.
            let context = if target == Span::unit(source_size) {
                Span::unit(source_size.checked_sub(1)?)
            } else {
                target
            };

            source.segment(context)
        });

        match segment {
            Some(extents) => ByLines::offload(error, ctx.referenced_with(move |input| (input, extents)).borrow_mut()),
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use core::num::NonZero;

    use crate::{
        prelude::{Label, oneshot},
        render::{
            RenderMut,
            textual::{
                Present, Textual,
                offload::fancy::{Colored, FancySettings, LayoutSettings, charset::Charset},
            },
        },
    };

    use aldebaran_baseline::prelude::Span;

    use super::Fancy;

    /// Render with a distinct title and a predictable, uncolored gutter.
    fn render_target(source: &str, target: Span) -> String {
        let report = oneshot::Oneshot::new("report title", source, Label::new("annotation detail", target));
        let mut output = String::new();
        let mut renderer = Textual::<'_, _, Fancy, _>::new(&mut output);
        let settings = FancySettings::tuple((Colored::No, LayoutSettings::marginal(), Charset::UNICODE));

        renderer
            .render_mut_with_input(Present::from_input(settings), &report)
            .expect("render diagnostic");
        output
    }

    #[test]
    fn eof_without_terminator_marks_after_the_final_scalar() {
        for (source, expected) in [("abc", "1│ abc\n │    ^\n"), ("é🦀", "1│ é🦀\n │    ^\n")] {
            let output = render_target(source, Span::unit(source.len()));
            assert!(output.contains(expected), "incorrect EOF marker placement: {output}");
            assert!(
                output.contains("[^]: annotation detail"),
                "annotation must appear outside the heading: {output}"
            );
        }
    }

    #[test]
    fn eof_after_lf_or_crlf_marks_a_separate_display_row() {
        for source in ["abc\n", "abc\r\n", "é🦀\n", "é🦀\r\n", "\n", "\r\n"] {
            let output = render_target(source, Span::unit(source.len()));
            assert!(
                output.contains("2│ \n │ ^\n │   [^]: annotation detail\n"),
                "incorrect terminal row: {output}"
            );
            assert!(!output.contains("│    ^"), "the marker must not attach to prior content: {output}");
            if source.starts_with("abc") {
                assert!(output.contains("1│ abc\n"), "final source context is missing: {output}");
            }
            if source.starts_with('é') {
                assert!(output.contains("1│ é🦀\n"), "UTF-8 context is missing: {output}");
            }
        }
    }

    #[test]
    fn wrapped_eof_preserves_the_final_text_and_utf8_boundaries() {
        for prefix_length in [139, 140] {
            let source = "a".repeat(prefix_length) + "é🦀";
            let output = render_target(&source, Span::unit(source.len()));

            if prefix_length == 139 {
                assert!(
                    output.contains(&("1│ ".to_owned() + &"a".repeat(139) + "é\n")),
                    "split UTF-8 phase: {output}"
                );
                assert!(output.contains(" │ 🦀\n │   ^\n"), "wrapped EOF marker is misplaced: {output}");
            } else {
                assert!(
                    output.contains(" │ é🦀\n │    ^\n"),
                    "final phase repeated the line prefix: {output}"
                );
            }
            assert!(
                output.contains("[^]: annotation detail"),
                "wrapped annotation disappeared: {output}"
            );
        }
    }

    #[test]
    fn terminal_boundary_and_content_annotations_keep_their_own_positions() {
        use aldebaran_baseline::prelude::RefNonempty;

        let source = "first\nabc\r\n";
        let content = Label::new("content detail", Span::unit(7));
        let terminal = [Label::new("terminal detail", Span::unit(source.len()))];
        let report = oneshot::many(source, "report title", RefNonempty::multiple(&content, &terminal));
        let mut output = String::new();
        let mut renderer = Textual::<'_, _, Fancy, _>::new(&mut output);
        let settings = FancySettings::tuple((Colored::No, LayoutSettings::marginal(), Charset::UNICODE));

        renderer
            .render_mut_with_input(Present::from_input(settings), &report)
            .expect("render multiple annotations");
        assert!(output.contains("2│ abc\n │  ^\n"), "content annotation moved: {output}");
        assert!(output.contains("3│ \n │ *\n"), "terminal annotation moved: {output}");
        assert!(output.contains("[^]: content detail"), "content annotation is missing: {output}");
        assert!(output.contains("[*]: terminal detail"), "terminal annotation is missing: {output}");
    }

    #[test]
    fn empty_and_wholly_out_of_bounds_targets_have_only_a_heading() {
        for (source, target) in [
            ("", Span::unit(0)),
            ("abc", Span::unit(4)),
            ("abc\n", Span::unit(5)),
            ("abc", Span::new(3, NonZero::new(2).expect("positive length"))),
            ("abc\n", Span::new(4, NonZero::new(2).expect("positive length"))),
            ("abc", Span::END_OF_WORLD),
        ] {
            let output = render_target(source, target);
            assert!(output.contains("report title"), "heading is missing: {output}");
            assert!(
                !output.contains("annotation detail"),
                "invalid target acquired an annotation: {output}"
            );
            assert!(!output.contains('│'), "invalid target acquired source context: {output}");
        }
    }

    #[test]
    fn partially_out_of_bounds_targets_clip_to_source_content() {
        let output = render_target("abc", Span::new(2, NonZero::<usize>::MAX));
        assert!(output.contains("1│ abc\n │   ^\n"), "incorrect clipped marker: {output}");
        assert!(output.contains("[^]: annotation detail"), "clipped annotation is missing: {output}");
    }

    #[test]
    fn gutter_fits_large_absolute_line_numbers_and_digit_transitions() {
        let source = "x\n".repeat(1001);
        let output = render_target(&source, Span::unit(1998));
        assert!(output.contains("1000│ x\n    │ ^\n"), "single-line gutter is too narrow: {output}");
        assert!(!output.contains("1001│"), "selection continued past its end: {output}");
        let output = render_target(&source, Span::new(1996, NonZero::new(4).expect("positive length")));
        assert!(output.contains(" 999│ x\n    │ ^\n"), "shorter numbers must align: {output}");
        assert!(output.contains("1000│ x\n    │ ^\n"), "multiline gutter is too narrow: {output}");
        let source = "x\n".repeat(9);
        let output = render_target(&source, Span::unit(source.len()));
        assert!(
            output.contains(" 9│ x\n10│ \n  │ ^\n"),
            "terminal row must contribute to gutter width: {output}"
        );
    }

    #[test]
    fn unicode_source_marker_uses_rendered_width() {
        let source = "let résumé = 1;\n";
        let start = source.find("résumé").expect("unicode binding is present");
        let length = NonZero::new("résumé".len()).expect("unicode binding is nonempty");
        let annotation = Label::new("unicode binding", Span::new(start, length));
        let report = oneshot::single(source, annotation);
        let mut output = String::new();
        let mut renderer = Textual::<'_, _, Fancy, _>::new(&mut output);
        let settings = FancySettings::tuple((Colored::No, LayoutSettings::standard(), Charset::UNICODE));

        renderer
            .render_mut_with_input(Present::from_input(settings), &report)
            .expect("failed to render unicode diagnostic");

        assert!(output.contains("let résumé = 1;"));
        assert!(output.contains("│     ^^^^^^"));
        assert!(!output.contains("│     ^^^^^^^^"));
    }

    #[test]
    fn markers_follow_terminal_cells_for_wide_combining_and_tab_text() {
        for (source, target, expected) in [
            ("中x", Span::unit(3), "1│ 中x\n │   ^\n"),
            ("e\u{301}x", Span::unit(3), "1│ e\u{301}x\n │  ^\n"),
            ("a\tx", Span::unit(2), "1│ a   x\n │     ^\n"),
            ("中\tx", Span::unit(4), "1│ 中  x\n │     ^\n"),
            ("中x", Span::new(0, NonZero::new(3).expect("wide scalar")), "1│ 中x\n │ ^^\n"),
            ("a\tx", Span::unit(1), "1│ a   x\n │  ^^^\n"),
            ("👩‍💻x", Span::unit("👩‍💻".len()), "1│ 👩‍💻x\n │   ^\n"),
        ] {
            let output = render_target(source, target);
            assert!(output.contains(expected), "incorrect terminal alignment {output}");
        }
    }

    #[test]
    fn wrapping_uses_cells_and_retains_combining_marks() {
        for (source, expected) in [
            (
                "中".repeat(71),
                "1│ ".to_owned() + &"中".repeat(70) + "\n │ " + &"^".repeat(140) + "\n",
            ),
            (
                "é".repeat(141),
                "1│ ".to_owned() + &"é".repeat(140) + "\n │ " + &"^".repeat(140) + "\n",
            ),
            ("a".repeat(139) + "e\u{301}x", "1│ ".to_owned() + &"a".repeat(139) + "e\u{301}\n"),
        ] {
            let output = render_target(&source, Span::ALL);
            assert!(output.contains(&expected), "incorrect cell wrapping {output}");
        }
    }

    #[test]
    fn equal_endpoint_messages_preserve_caller_order() {
        use aldebaran_baseline::prelude::RefNonempty;

        let first = Label::new("first detail", Span::unit(0));
        let rest = [
            Label::new("second detail", Span::unit(0)),
            Label::new("third detail", Span::unit(0)),
            Label::new("fourth detail", Span::unit(0)),
        ];
        let report = oneshot::many("x", "report title", RefNonempty::multiple(&first, &rest));
        let mut output = String::new();
        let mut renderer = Textual::<'_, _, Fancy, _>::new(&mut output);
        let settings = FancySettings::tuple((Colored::No, LayoutSettings::marginal(), Charset::UNICODE));

        renderer
            .render_mut_with_input(Present::from_input(settings), &report)
            .expect("render equal endpoints");

        let positions: Vec<_> = ["first detail", "second detail", "third detail", "fourth detail"]
            .map(|message| output.find(message).expect("annotation message"))
            .into_iter()
            .collect();
        assert!(
            positions.iter().zip(positions.iter().skip(1)).all(|(first, second)| first < second),
            "annotation order changed {output}"
        );
    }

    #[test]
    fn multiline_annotation_renders_all_selected_content_and_its_markers() {
        let output = render_target("snicker\n dog bone\n", Span::ALL);
        assert!(
            output.contains("1│ snicker\n │ ^^^^^^^\n"),
            "first line markers are missing: {output}"
        );
        assert!(
            output.contains("2│  dog bone\n │ ^^^^^^^^^\n"),
            "final line markers are missing: {output}"
        );
        assert_eq!(output.matches("[^]: annotation detail").count(), 1);
    }
}
