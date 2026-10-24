use alloc::{string::String, vec::Vec};
use core::{fmt, mem, num::NonZero};

use super::{Cell, LayoutOptions, SourceText};
use crate::visualize::Visualize;

#[test]
fn ordinary_rows_borrow_input_and_reused_buffers_release_previous_line_lifetimes() {
    let layout_options = LayoutOptions::new(NonZero::new(4).expect("columns"), NonZero::new(4).expect("tabs"));
    let mut text_layout = SourceText::new(usize::MIN).layout(layout_options);

    for source in ["中e\u{301}x", "a\tx", "\0x", "e\u{301}中"] {
        let source = String::from(source);

        let (empty, mut layout) = text_layout.reused(SourceText::borrowed(&source, 20), layout_options);

        drop(empty);

        let row = layout.next_row().expect("first row");

        assert_eq!(row.source_range().start, 20, "source coordinates must retain their origin");

        if !source.contains(['\t', '\0']) {
            assert_eq!(row.text().as_ptr(), source.as_ptr(), "ordinary rows must borrow their source bytes");
        }

        while layout.next_row().is_some() {}

        let (input, reused) = layout.reused(SourceText::new(usize::MIN), layout_options);

        let cleared = input.cleared(usize::MIN);

        assert_eq!(cleared.source_end, usize::MIN, "clearing must release previous coordinates");

        text_layout = reused;
    }
}

#[test]
fn cells_share_whole_graphemes_and_preserve_utf8_source_coordinates() {
    let source = "e\u{301}界👨‍👩‍👧‍👦🇧🇷!";
    let mut layout = SourceText::from(source).layout(LayoutOptions::new(
        NonZero::new(80).expect("columns"),
        NonZero::new(4).expect("tabs"),
    ));
    let row = layout.next_row().expect("one row");

    assert_eq!(row.text(), source);
    assert_eq!(row.cells().len(), 8);
    assert_eq!(row.source_range(), usize::MIN..source.len());
    assert_eq!(
        row.columns_for(1..3),
        Some(usize::MIN..1),
        "an accent selects its whole base cluster"
    );
    assert_eq!(row.columns_for(4..5), Some(1..3), "even part of a wide scalar selects both cells");

    for cell in row.cells() {
        assert!(row.text().get(cell.text_bytes()).is_some());
        assert!(source.get(cell.source_range()).is_some());
    }

    assert_eq!(
        row.cells().get(1).map(|cell| cell.source_range()),
        row.cells().get(2).map(|cell| cell.source_range())
    );
    assert_eq!(
        row.columns_for(source.len()..source.len() + 1),
        None,
        "EOF is separate from source content"
    );
    assert_eq!(row.columns_for(1..1), None, "empty selections contain no source");
    assert!(layout.next_row().is_none());
    assert!(layout.next_row().is_none(), "exhaustion remains stable");
}

#[test]
fn wrapping_never_splits_combining_emoji_variation_or_regional_clusters() {
    for cluster in ["e\u{301}", "👨‍👩‍👧‍👦", "👩🏽‍💻", "🇧🇷", "❤️"] {
        for budget in 1..=4 {
            let source = alloc::format!("abc{cluster}xy");
            let mut layout = SourceText::from(source.as_str()).layout(LayoutOptions::new(
                NonZero::new(budget).expect("budget"),
                NonZero::new(4).expect("tabs"),
            ));
            let mut restored = String::new();
            let mut found_cluster = false;

            while let Some(row) = layout.next_row() {
                restored.push_str(row.text());
                found_cluster |= row.text().contains(cluster);

                assert!(
                    row.cells().len() <= budget || row.cells().iter().filter(|cell| !cell.text_bytes().is_empty()).count() == 1,
                    "only an indivisible cluster may exceed the budget"
                );
            }

            assert_eq!(restored, source);
            assert!(found_cluster, "cluster split: {cluster:?}, budget {budget}");
        }
    }
}

#[test]
fn tabs_controls_and_wrapping_use_the_same_cell_map() {
    let mut layout = SourceText::from("界\tX\u{1b}").layout(LayoutOptions::new(
        NonZero::new(4).expect("columns"),
        NonZero::new(4).expect("tabs"),
    ));
    let first = layout.next_row().expect("wide character and tab");

    assert_eq!(first.text(), "界  ");
    assert_eq!(first.cells().len(), 4);
    assert_eq!(first.columns_for(3..4), Some(2..4), "tab selects its expanded cells");

    let second = layout.next_row().expect("remaining text and control");

    assert_eq!(second.text(), "X�");
    assert_eq!(second.columns_for(5..6), Some(1..2));
    assert!(layout.next_row().is_none());

    let mut layout = SourceText::from("abc\tX").layout(LayoutOptions::new(
        NonZero::new(3).expect("columns"),
        NonZero::new(8).expect("tabs"),
    ));
    let mut string_list = Vec::new();

    while let Some(row) = layout.next_row() {
        string_list.push(String::from(row.text()));
    }

    assert_eq!(
        string_list,
        ["abc", "        ", "X"],
        "a wrapped tab is measured from its new row origin"
    );
}

#[test]
fn component_visualizations_retain_source_units_independent_of_utf8_length() {
    let mut text = SourceText::new(10);

    text.push(NonZero::<usize>::MIN, &0xff_u8)
        .expect("one replacement for a source byte");
    text.push(NonZero::<usize>::MIN, &b'X').expect("one ASCII byte");

    let mut layout = text.layout(LayoutOptions::new(
        NonZero::new(80).expect("columns"),
        NonZero::new(4).expect("tabs"),
    ));
    let row = layout.next_row().expect("byte source row");

    assert_eq!(row.source_range(), 10..12);
    assert_eq!(row.text(), "�X");
    assert_eq!(row.columns_for(10..11), Some(usize::MIN..1));
    assert_eq!(row.columns_for(11..12), Some(1..2));
}

#[test]
fn segmentation_spans_separate_component_writes() {
    let source = "👩🏽‍💻e\u{301}";
    let mut text = SourceText::new(20);

    for character in source.chars() {
        text.push(NonZero::new(character.len_utf8()).expect("character width"), &character)
            .expect("component");
    }

    let mut layout = text.layout(LayoutOptions::new(
        NonZero::new(80).expect("columns"),
        NonZero::new(4).expect("tabs"),
    ));
    let row = layout.next_row().expect("row");

    assert_eq!(row.cells().len(), 3);
    assert_eq!(row.source_range(), 20..20 + source.len());
    assert_eq!(
        row.columns_for(24..28),
        Some(usize::MIN..2),
        "skin-tone component belongs to the complete emoji"
    );
}

#[test]
fn empty_and_zero_width_rows_retain_their_source_anchor() {
    let layout_options = LayoutOptions::new(NonZero::<usize>::MIN, NonZero::new(4).expect("tabs"));
    let mut empty = SourceText::new(13).layout(layout_options);
    let row = empty.next_row().expect("empty logical line retains one row");

    assert_eq!(row.source_range(), 13..13);
    assert!(row.cells().is_empty());
    assert!(row.text().is_empty());
    assert!(empty.next_row().is_none());

    let mut combining = SourceText::from("\u{301}").layout(layout_options);
    let row = combining.next_row().expect("zero-width cluster");

    assert!(row.cells().is_empty());
    assert_eq!(row.columns_for(usize::MIN..2), Some(usize::MIN..usize::MIN));
}

/// A visualization that fails after writing part of its output.
struct Failing;

impl Visualize for Failing {
    fn visualize<W>(&self, writer: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        writer.write_str("partial")?;
        Err(fmt::Error)
    }
}

#[test]
fn failed_component_appends_are_atomic_and_overflow_does_not_advance_source() {
    let layout_options = LayoutOptions::new(NonZero::new(80).expect("columns"), NonZero::new(4).expect("tabs"));
    let mut text = SourceText::from("x");

    assert!(text.push(NonZero::<usize>::MIN, &Failing).is_err());
    text.push(NonZero::<usize>::MIN, &'y').expect("append after failed visualization");

    let mut layout = text.layout(layout_options);
    let row = layout.next_row().expect("row");

    assert_eq!(row.text(), "xy");
    assert_eq!(row.source_range(), usize::MIN..2);

    let mut text = SourceText::new(usize::MAX);

    assert!(text.push(NonZero::<usize>::MIN, &'x').is_err());

    let mut layout = text.layout(layout_options);
    let row = layout.next_row().expect("empty row");

    assert_eq!(row.source_range(), usize::MAX..usize::MAX);
    assert!(row.text().is_empty());
}

#[test]
fn combining_input_extends_the_existing_cell_without_contributor_storage() {
    let mut layout = SourceText::from("\t\u{301}").layout(LayoutOptions::new(
        NonZero::new(8).expect("columns"),
        NonZero::new(4).expect("tabs"),
    ));
    let row = layout.next_row().expect("one row");
    let last = row.cells().last().expect("last expanded tab cell");

    assert_eq!(row.text(), "    \u{301}");
    assert_eq!(row.cells().len(), 4);
    assert_eq!(last.source_range(), usize::MIN..3, "the existing cell retains both source portions");
    assert_eq!(row.columns_for(usize::MIN..1), Some(usize::MIN..4));
    assert_eq!(row.columns_for(1..3), Some(3..4));

    assert_eq!(mem::size_of::<Cell>(), 4 * mem::size_of::<usize>());
    assert!(!mem::needs_drop::<Cell>(), "cells own no heap storage");
}

#[test]
fn zero_width_input_stays_with_an_oversized_tab_cell() {
    let layout_options = LayoutOptions::new(NonZero::new(3).expect("columns"), NonZero::new(4).expect("tabs"));
    let mut layout = SourceText::from("\t\u{301}").layout(layout_options);
    let row = layout.next_row().expect("expanded tab row");

    assert_eq!(row.text(), "    \u{301}");
    assert_eq!(row.columns_for(1..3), Some(3..4));
    assert!(layout.next_row().is_none());
}

#[test]
fn regional_indicator_pairs_keep_segmentation_context_across_rows() {
    let mut layout = SourceText::from("🇦🇧🇨🇩🇪🇫").layout(LayoutOptions::new(NonZero::new(2).expect("one flag"), NonZero::<usize>::MIN));

    for (index, flag) in ["🇦🇧", "🇨🇩", "🇪🇫"].into_iter().enumerate() {
        let row = layout.next_row().expect("paired regional indicators");

        assert_eq!(row.text(), flag);
        assert_eq!(row.source_range(), index * 8..index * 8 + 8);
        assert_eq!(row.cells().len(), 2);
        assert_eq!(row.columns_for(index * 8 + 4..index * 8 + 8), Some(usize::MIN..2));
    }

    assert!(layout.next_row().is_none());
}

#[test]
fn invisible_components_preserve_source_boundaries_between_wrapped_rows() {
    let mut text = SourceText::new(10);

    text.push(NonZero::<usize>::MIN, &"a").expect("first component");
    text.push(NonZero::new(3).expect("source width"), &"").expect("invisible component");
    text.push(NonZero::<usize>::MIN, &"b").expect("second component");
    text.push(NonZero::new(2).expect("source width"), &"")
        .expect("invisible trailing component");

    let mut layout = text.layout(LayoutOptions::new(NonZero::<usize>::MIN, NonZero::<usize>::MIN));
    let first = layout.next_row().expect("first row");

    assert_eq!(first.text(), "a");
    assert_eq!(first.source_range(), 10..11);
    assert_eq!(first.columns_for(11..14), None);

    let second = layout.next_row().expect("second row");

    assert_eq!(second.text(), "b");
    assert_eq!(second.source_range(), 14..17);
    assert_eq!(second.columns_for(14..15), Some(usize::MIN..1));
    assert_eq!(second.columns_for(11..14), None);
    assert!(layout.next_row().is_none());
}

#[test]
fn large_column_and_tab_settings_do_not_reserve_unneeded_output() {
    let maximum = NonZero::new(usize::MAX).expect("maximum is nonzero");
    let layout_options = LayoutOptions::new(maximum, maximum);

    for source in ["", "x"] {
        let mut layout = SourceText::from(source).layout(layout_options);
        let row = layout.next_row().expect("logical source line");

        assert_eq!(row.text(), source);
        assert_eq!(row.cells().len(), source.len());
        assert_eq!(row.source_range(), usize::MIN..source.len());
        assert!(layout.next_row().is_none());
    }
}

#[test]
fn cloning_with_pending_graphemes_preserves_projection_and_row_state() {
    let mut source_text = SourceText::new(10);

    for (width, text) in [(1, "a"), (7, "👨‍👩‍👧‍👦"), (2, "\t"), (1, "\u{301}"), (1, "x")] {
        let width = NonZero::new(width).expect("source component width");

        source_text.push(width, &text).expect("component visualization");
    }

    let column_count = NonZero::new(2).expect("column budget");
    let tab_width = NonZero::new(4).expect("tab interval");
    let layout_options = LayoutOptions::new(column_count, tab_width);
    let mut text_layout = source_text.layout(layout_options);
    let first = text_layout.next_row().expect("row preceding pending family");

    assert_eq!(first.text(), "a");
    assert_eq!(first.source_range(), 10..11);

    let cloned_layout = text_layout.clone();

    for mut text_layout in [text_layout, cloned_layout] {
        for (text, source_range, cell_count) in [("👨‍👩‍👧‍👦", 11..18, 2), ("    \u{301}", 18..21, 4), ("x", 21..22, 1)] {
            let row = text_layout.next_row().expect("pending or subsequent row");

            assert_eq!(row.text(), text);
            assert_eq!(row.source_range(), source_range);
            assert_eq!(row.cells().len(), cell_count);
        }

        assert!(text_layout.next_row().is_none());
        assert!(text_layout.next_row().is_none(), "finished traversal stays exhausted");
    }
}
