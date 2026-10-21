//! Annotation clipping, overlap ordering, and gap-coordinate expectations.

use alloc::vec::Vec;
use core::num::NonZero;

use aldebaran_baseline::prelude::{Span, Spanned};

use crate::line::{LineExtent, LineSegmented, SourceLines};

/// Nonempty source interval specified by an inclusive start and exclusive end.
fn span(start: usize, end: usize) -> Span {
    Span::new(start, NonZero::new(end - start).expect("nonempty test interval"))
}

#[test]
fn later_lines_clip_crossing_spans_and_exclude_terminators() {
    let source = "first\r\nsecond\r\n";
    let extents = source.extents().expect("source lines");
    let line = extents.lines().nth(1).expect("second line");
    let annotations = [span(5, 8), span(11, 15), span(13, 15)];
    let pieces: Vec<_> = LineSegmented::segmented(&line, annotations.into_iter()).collect();

    assert_eq!(pieces.iter().map(|piece| *piece.view()).collect::<Vec<_>>(), ["s", "eco", "nd"]);
    assert_eq!(
        pieces.iter().map(Spanned::span).collect::<Vec<_>>(),
        [span(5, 8), span(8, 11), span(11, 15)]
    );
    assert_eq!(
        pieces.iter().filter_map(|piece| piece.data().copied()).collect::<Vec<_>>(),
        annotations[..2]
    );
}

#[test]
fn gaps_preserve_pending_annotations_and_overlaps_cover_their_union() {
    let source = "0123456789abcdef";
    let extents = source.extents().expect("source lines");
    let line = extents.lines().next().expect("first line");
    let annotations = [span(4, 12), span(4, 8), span(6, 7), span(10, 14)];
    let mut segments = LineSegmented::segmented(&line, annotations.into_iter());
    let pieces: Vec<_> = segments.by_ref().collect();

    assert_eq!(
        pieces.iter().map(|piece| *piece.view()).collect::<Vec<_>>(),
        ["0123", "456789ab", "4567", "6", "abcd", "ef"]
    );
    assert_eq!(
        pieces.iter().filter_map(|piece| piece.data().copied()).collect::<Vec<_>>(),
        annotations
    );
    assert_eq!(pieces.last().map(Spanned::span), Some(span(14, 16)));
    assert!(segments.next().is_none());
    assert!(segments.next().is_none());
}

#[test]
fn unordered_byte_annotations_keep_duplicate_starts_after_clipping() {
    let source = b"first\nsecond".as_slice();
    let extents = source.extents().expect("source lines");
    let line = extents.lines().nth(1).expect("second line");
    let annotations = [span(9, 10), span(6, 8), span(5, 7)];
    let pieces: Vec<_> = LineSegmented::segmented(&line, annotations.into_iter()).collect();

    assert_eq!(
        pieces.iter().map(|piece| *piece.view()).collect::<Vec<_>>(),
        [b"se".as_slice(), b"s", b"c", b"o", b"nd"]
    );
    assert_eq!(
        pieces.iter().filter_map(|piece| piece.data().copied()).collect::<Vec<_>>(),
        [annotations[1], annotations[2], annotations[0]]
    );
}

#[test]
fn invalid_utf8_windows_do_not_hide_later_annotations() {
    let source = "é中x";
    let extents = source.extents().expect("source lines");
    let line = extents.lines().next().expect("first line");
    let annotations = [Span::unit(1), span(2, 5), Span::unit(5)];
    let pieces: Vec<_> = LineSegmented::segmented(&line, annotations.into_iter()).collect();

    assert_eq!(pieces.iter().map(|piece| *piece.view()).collect::<Vec<_>>(), ["é", "中", "x"]);
    assert_eq!(
        pieces.iter().filter_map(|piece| piece.data().copied()).collect::<Vec<_>>(),
        annotations[1..]
    );
}
