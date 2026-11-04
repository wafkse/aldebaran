//! Lays out source visualizations as rows of terminal cells with source provenance.
//!
//! See [`SourceText`], [`TextLayout`], [`TextRow`], and [`Cell`] for further information.
//!
//! Rows share reusable output storage, so advancing the layout requires releasing
//! the preceding row's borrow. Cell storage grows with the widest displayed row
//! rather than the length of the logical line. Unmodified row text borrows the
//! input directly, while tabs and control replacements use owned text storage.
//!
//! This module expects a logical line rather than splitting physical lines.
//! Tabs expand from each row's origin, other control scalars become replacement
//! characters, and ordinary text follows the non-CJK `unicode-width` policy.
//!
//! ```
//! use core::num::NonZero;
//! use aldebaran_print::layout::{LayoutOptions, SourceText};
//!
//! let layout_options = LayoutOptions::new(NonZero::new(80).expect("columns"), NonZero::new(4).expect("tabs"));
//! let mut layout = SourceText::from("e\u{301}界").layout(layout_options);
//! let row = layout.next_row().expect("logical line");
//! assert_eq!(row.cells().len(), 3);
//! assert_eq!(row.columns_for(1..3), Some(0..1));
//! ```

use alloc::{borrow::Cow, string::String, vec::Vec};
use core::{fmt, num::NonZero, ops::Range};

use crate::visualize::Visualize;

mod grapheme;
mod projection;
mod row;
mod terminal;

use self::{
    grapheme::GraphemeTraversal,
    projection::SourceProjection,
    row::{RowBuffer, RowBuilder, RowTraversal},
    terminal::TerminalGrapheme,
};

/// Associates a terminal column with its source extent and row text.
///
/// A cell's text range indexes UTF-8 bytes in its containing [`TextRow`]. Wide
/// text occupies a leading cell followed by continuation cells with empty text
/// ranges, so printing the leading range emits the text without duplication.
/// The continuation cells retain source provenance for annotation projection.
///
/// Zero-width input can extend an existing cell's source and text ranges. Each
/// cell stores these ranges directly and owns no text or contributor storage.
#[derive(Debug, Clone, PartialEq, Eq)]
// NOTE(invariant): TextByteRange Within ContainingRow.Text && SourceRange Within ContainingRow.SourceRange
pub struct Cell {
    /// Original source coordinates represented by this cell.
    source_range: Range<usize>,

    /// Row-relative bytes emitted by this cell; empty for a continuation.
    text_byte_range: Range<usize>,
}

impl Cell {
    /// Returns the original source extent represented by this column.
    ///
    /// These coordinates use the units of the [`SourceText`], which need not be
    /// UTF-8 bytes for a component visualization.
    #[inline]
    pub fn source_range(&self) -> Range<usize> {
        let Self { source_range, .. } = self;

        source_range.clone()
    }

    /// Returns the byte range to print from the containing [`TextRow::text`].
    ///
    /// The range is empty for a wide-text continuation cell.
    #[inline]
    pub fn text_bytes(&self) -> Range<usize> {
        let Self { text_byte_range, .. } = self;

        text_byte_range.clone()
    }
}

/// Relates a component's original source extent to its visualization bytes.
///
/// A component can occupy source units without emitting text, or emit several
/// UTF-8 bytes for a narrower source extent. These ranges retain that relation
/// independently of the grapheme boundaries used for display.
#[derive(Debug, Clone, PartialEq, Eq)]
// NOTE(invariant): SourceRange.IsNonEmpty && TextByteRange Within Input.Text
struct SourceFragment {
    /// Coordinates in the original source's units.
    source_range: Range<usize>,

    /// UTF-8 bytes in the input visualization buffer.
    text_byte_range: Range<usize>,
}

/// Source visualization together with the coordinates needed to select its content.
///
/// Borrowed UTF-8 input retains its original bytes without copying them. For
/// component input, [`Self::push`] appends each visualization while recording
/// its extent in the original source units. Graphemes may span component writes,
/// so source extents remain independent of display segmentation.
///
/// This input is retained by [`TextLayout`] while rows are lent. Owned text and
/// provenance storage can be reused for another line through [`Self::cleared`].
#[derive(Debug, Clone, PartialEq, Eq)]
// NOTE(invariant): Ordered(SourceFragmentList) && (SourceFragmentList.IsEmpty => SourceEnd - SourceStart = Text.Utf8Length)
pub struct SourceText<'source> {
    /// Visualization before terminal controls are normalized.
    text: Cow<'source, str>,

    /// Temporary source fragments; empty for an identity UTF-8 source view.
    source_fragment_list: Vec<SourceFragment>,

    /// Exclusive original source endpoint.
    source_end: usize,

    /// Initial source coordinate, including for an empty line.
    source_start: usize,
}

impl<'source> SourceText<'source> {
    /// Borrows text whose first byte has source coordinate `source_start`.
    ///
    /// Each subsequent UTF-8 byte keeps its offset relative to that coordinate,
    /// including when a source target starts within a scalar or grapheme.
    ///
    /// # Panics
    ///
    /// Panics if `source_start + text.len()` exceeds [`usize::MAX`].
    #[inline]
    pub const fn borrowed(text: &'source str, source_start: usize) -> Self {
        let source_end = source_start.checked_add(text.len()).expect("source endpoint fits in usize");
        let text = Cow::Borrowed(text);
        let source_fragment_list = Vec::new();

        Self {
            text,
            source_fragment_list,
            source_start,
            source_end,
        }
    }

    /// Returns empty input at `source_start`, retaining any owned storage.
    ///
    /// Consuming the previous input releases its source borrow. Borrowed text is
    /// discarded without copying, while owned text and provenance records keep
    /// their capacities for subsequent component writes.
    #[inline]
    pub fn cleared(self, source_start: usize) -> SourceText<'static> {
        let Self {
            text,
            mut source_fragment_list,
            ..
        } = self;

        let mut text = match text {
            Cow::Owned(text) => text,
            Cow::Borrowed(_) => String::new(),
        };

        text.clear();
        source_fragment_list.clear();

        let text = Cow::Owned(text);
        let source_end = source_start;

        SourceText {
            text,
            source_fragment_list,
            source_start,
            source_end,
        }
    }

    /// Creates empty component input whose source coordinates begin at `source_start`.
    ///
    /// Subsequent writes advance the source endpoint by their supplied source
    /// widths, independently of the bytes produced by visualization.
    #[inline]
    pub const fn new(source_start: usize) -> Self {
        let text = Cow::Owned(String::new());
        let source_fragment_list = Vec::new();
        let source_end = source_start;

        Self {
            text,
            source_fragment_list,
            source_end,
            source_start,
        }
    }

    /// Creates empty component input with space reserved for visualization bytes.
    ///
    /// `capacity` applies to the text buffer. Provenance records are allocated
    /// separately when component writes need an explicit coordinate mapping.
    #[inline]
    pub fn with_capacity(source_start: usize, capacity: usize) -> Self {
        let text = Cow::Owned(String::with_capacity(capacity));
        let source_fragment_list = Vec::new();
        let source_end = source_start;

        Self {
            text,
            source_fragment_list,
            source_end,
            source_start,
        }
    }

    /// Appends a component visualization with its width in original source units.
    ///
    /// The component's source extent starts at the current endpoint and advances
    /// by `source_width`, even if its visualization emits no text. Its UTF-8 byte
    /// length does not determine its source width or terminal cell count.
    ///
    /// # Errors
    ///
    /// Returns [`fmt::Error`] if the source endpoint would overflow or the
    /// visualization fails. Text and source coordinates accepted by earlier
    /// writes are preserved, including when the failing visualization wrote
    /// part of its output.
    #[inline]
    pub fn push<V>(&mut self, source_width: NonZero<usize>, value: &V) -> fmt::Result
    where
        V: Visualize + ?Sized,
    {
        let Self {
            text,
            source_fragment_list,
            source_end,
            source_start,
        } = self;

        let next_source_end = source_end.checked_add(source_width.get()).ok_or(fmt::Error)?;
        let start = text.len();
        let buffer = text.to_mut();

        value.visualize(buffer).inspect_err(|_| buffer.truncate(start))?;

        let identity = source_fragment_list.is_empty()
            && buffer
                .get(start..)
                .is_some_and(|text| text.len() == source_width.get() && text.chars().count() == 1);

        match (source_fragment_list.as_slice(), identity, start) {
            ([], true, _) => {}
            ([], false, usize::MIN) | ([_, ..], _, _) => {
                let source_range = *source_end..next_source_end;
                let text_byte_range = start..buffer.len();

                source_fragment_list.push(SourceFragment {
                    source_range,
                    text_byte_range,
                });
            }
            ([], false, _) => {
                source_fragment_list.extend(buffer.get(..start).expect("original text boundary").char_indices().map(
                    |(offset, character)| {
                        let source_range = *source_start + offset..*source_start + offset + character.len_utf8();
                        let text_byte_range = offset..offset + character.len_utf8();

                        SourceFragment {
                            source_range,
                            text_byte_range,
                        }
                    },
                ));

                let source_range = *source_end..next_source_end;
                let text_byte_range = start..buffer.len();

                source_fragment_list.push(SourceFragment {
                    source_range,
                    text_byte_range,
                });
            }
        }

        *source_end = next_source_end;

        Ok(())
    }

    /// Transfers this input to a layout using the supplied row and tab policy.
    #[inline]
    pub fn layout(self, layout_options: LayoutOptions) -> TextLayout<'source> {
        TextLayout::new(self, layout_options)
    }
}

impl<'source> From<&'source str> for SourceText<'source> {
    #[inline]
    fn from(source: &'source str) -> Self {
        Self::borrowed(source, usize::MIN)
    }
}

/// Selects the cell budget of wrapped rows and the interval between tab stops.
///
/// Both values are measured in terminal cells. The row budget permits an
/// indivisible first display unit to exceed it, while tab stops restart from
/// the origin of each wrapped row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// NOTE(invariant): None.
pub struct LayoutOptions {
    /// Available cells in a wrapped row.
    column_count: NonZero<usize>,

    /// Tab-stop interval from the row origin.
    tab_width: NonZero<usize>,
}

impl LayoutOptions {
    /// Sets the nonzero row budget and tab-stop interval, in terminal cells.
    #[inline]
    pub const fn new(column_count: NonZero<usize>, tab_width: NonZero<usize>) -> Self {
        Self { column_count, tab_width }
    }
}

/// Borrows a displayed row and the source provenance of its terminal cells.
///
/// Its text is ready to print, with tabs expanded and control scalars replaced.
/// Cell indices are row-relative terminal columns, while source ranges retain
/// the original coordinates supplied by [`SourceText`].
///
/// The row owns no allocation. Its borrow prevents the originating layout from
/// advancing or replacing its input while the row remains in use.
#[derive(Debug, Clone, PartialEq, Eq)]
// NOTE(invariant): ForAll(Cell In CellList): Cell.TextByteRange Within Text && Cell.SourceRange Within SourceRange
pub struct TextRow<'layout> {
    /// Original source extent, including zero-width input at an empty anchor.
    source_range: Range<usize>,

    /// The shared text buffer's row slice.
    text: &'layout str,

    /// The shared cell buffer's row slice.
    cell_list: &'layout [Cell],
}

impl<'layout> TextRow<'layout> {
    /// Returns the original source extent represented by this row.
    ///
    /// This can include input with no terminal width. An empty input yields an
    /// empty range at its configured source coordinate.
    #[inline]
    pub fn source_range(&self) -> Range<usize> {
        let Self { source_range, .. } = self;

        source_range.clone()
    }

    /// Returns the displayed text without a gutter or trailing row delimiter.
    #[inline]
    pub const fn text(&self) -> &'layout str {
        let Self { text, .. } = self;

        text
    }

    /// Returns the cell records in terminal-column order.
    ///
    /// Continuation cells count toward the row width but do not repeat the text
    /// of their leading cell.
    #[inline]
    pub const fn cells(&self) -> &'layout [Cell] {
        let Self { cell_list, .. } = self;

        cell_list
    }

    /// Projects a source target to the terminal columns of its intersecting cells.
    ///
    /// The returned range covers the first through last selected columns. A
    /// target can select part of a scalar or grapheme, and wide-text provenance
    /// selects its continuation cells as well. Targets use the input's source
    /// units rather than row text byte offsets.
    ///
    /// Empty targets and targets outside the row return `None`. A row with no
    /// cells can yield an empty column range for a target spanning its source
    /// anchor. This method does not create a column for an EOF insertion.
    #[inline]
    pub fn columns_for(&self, target: Range<usize>) -> Option<Range<usize>> {
        let row_source_range = self.source_range();

        let is_nonempty_target = target.start < target.end;
        let overlaps_row = row_source_range.start < target.end && target.start < row_source_range.end;

        if is_nonempty_target && overlaps_row {
            let mut selected_cell_iter = self.cells().iter().enumerate().filter(|(_, cell)| {
                let cell_source_range = cell.source_range();

                cell_source_range.start < target.end && target.start < cell_source_range.end
            });

            match (selected_cell_iter.next(), self.cells()) {
                (Some((first_cell_index, _)), _) => {
                    let last_cell_index = selected_cell_iter.next_back().map_or(first_cell_index, |(index, _)| index);

                    Some(first_cell_index..last_cell_index + 1)
                }
                (None, []) => Some(usize::MIN..usize::MIN),
                (None, [_, ..]) => None,
            }
        } else {
            None
        }
    }
}

/// Wraps a logical source line into borrowed rows with terminal-cell provenance.
///
/// Graphemes stay intact across row boundaries, including when the first visible
/// unit exceeds the configured cell budget. Zero-width input remains with its
/// preceding visible cell when available, and tabs are measured from the origin
/// of the row that accepts them.
///
/// Each [`Self::next_row`] call lends a view of the current output. Display and
/// annotation projection share its cell records, so querying several targets
/// does not repeat segmentation. Reusable output storage is retained when rows
/// advance and can also survive a change of input through [`Self::reused`].
#[derive(Debug, Clone)]
// NOTE(invariant): GraphemeTraversal.InputLength = Input.Text.Length && SourceProjection.FragmentIndex <= Input.SourceFragmentList.Length
pub struct TextLayout<'source> {
    /// Source visualization and original component coordinates.
    input: SourceText<'source>,

    /// Row width and tab-stop interval.
    layout_options: LayoutOptions,

    /// Segmentation context and unconsumed lookahead.
    grapheme_traversal: GraphemeTraversal,

    /// Forward projection into original source coordinates.
    source_projection: SourceProjection,

    /// Position and completion state of wrapped rows.
    row_traversal: RowTraversal,

    /// Reusable output storage, borrowed while a row is lent.
    row_buffer: RowBuffer,
}

impl<'source> TextLayout<'source> {
    /// Starts a layout for new input while retaining the previous output capacity.
    ///
    /// Returns the previous input alongside the new layout so its owned capacity
    /// can be recovered through [`SourceText::cleared`]. The new layout carries
    /// only the new input's lifetime. Segmentation, source projection, and row
    /// position all restart for the replacement input.
    #[inline]
    pub fn reused<'next>(self, input: SourceText<'next>, layout_options: LayoutOptions) -> (SourceText<'source>, TextLayout<'next>) {
        let Self {
            input: previous,
            row_buffer,
            ..
        } = self;

        let layout = TextLayout::prepared(input, layout_options, row_buffer);

        (previous, layout)
    }

    /// Creates a layout retaining `input` under the supplied display policy.
    ///
    /// Output capacity is reserved for the initial row rather than the entire
    /// logical line. Rows and their cell records borrow storage from the layout.
    #[inline]
    pub fn new(input: SourceText<'source>, layout_options: LayoutOptions) -> Self {
        let row_buffer = RowBuffer::default();

        Self::prepared(input, layout_options, row_buffer)
    }

    /// Associates fresh traversal state with input and reusable output storage.
    ///
    /// Existing output contents are discarded, but their capacity is retained
    /// and extended when the initial row reservation requires it.
    fn prepared(input: SourceText<'source>, layout_options: LayoutOptions, mut row_buffer: RowBuffer) -> Self {
        let SourceText { text, source_start, .. } = &input;

        let LayoutOptions { column_count, .. } = layout_options;

        let grapheme_traversal = GraphemeTraversal::new(text.len());
        let source_projection = SourceProjection::default();
        let row_traversal = RowTraversal::new(*source_start);

        row_buffer.prepare(text.len(), column_count);

        Self {
            input,
            layout_options,
            grapheme_traversal,
            source_projection,
            row_traversal,
            row_buffer,
        }
    }

    /// Lends the next displayed row until the layout is advanced again.
    ///
    /// Empty input yields exactly one empty row at its original source
    /// coordinate. After the final row, subsequent calls return `None`.
    /// The returned borrow keeps the row's text and cells valid by preventing
    /// further mutation of the layout while that row remains in use.
    ///
    /// ```compile_fail,E0499
    /// use core::num::NonZero;
    /// use aldebaran_print::layout::{LayoutOptions, SourceText};
    ///
    /// let layout_options = LayoutOptions::new(NonZero::<usize>::MIN, NonZero::<usize>::MIN);
    /// let mut layout = SourceText::from("ab").layout(layout_options);
    /// let first = layout.next_row().expect("first row");
    /// let second = layout.next_row().expect("second row");
    /// assert_eq!(first.text(), "a");
    /// assert_eq!(second.text(), "b");
    /// ```
    #[inline]
    pub fn next_row(&mut self) -> Option<TextRow<'_>> {
        let Self {
            input,
            layout_options,
            grapheme_traversal,
            source_projection,
            row_traversal,
            row_buffer,
        } = self;

        let &mut LayoutOptions { column_count, tab_width } = layout_options;

        let RowTraversal {
            next_source_start,
            is_exhausted,
        } = row_traversal;

        if *is_exhausted {
            None
        } else {
            let mut row_builder = RowBuilder::new(row_buffer, *next_source_start, grapheme_traversal.input_offset, column_count);

            loop {
                match grapheme_traversal.peek(&input.text) {
                    Ok(Some(input_byte_range)) => {
                        let text = input.text.get(input_byte_range.clone())?;
                        let source_range = source_projection.project(input, input_byte_range.clone())?;
                        let terminal_grapheme = TerminalGrapheme::new(text, tab_width, row_builder.column());

                        if row_builder.accepts(terminal_grapheme.width()) {
                            let end = input_byte_range.end;

                            row_builder.push(&input.text, input_byte_range, source_range, terminal_grapheme)?;
                            grapheme_traversal.input_offset = end;
                        } else {
                            *next_source_start = source_range.start;
                            break;
                        }
                    }
                    Ok(None) => {
                        *is_exhausted = true;
                        row_builder.source_range.end = input.source_end;
                        break;
                    }
                    Err(_) => return None,
                }
            }

            row_builder.finish(&input.text)
        }
    }
}

#[cfg(test)]
mod tests {
    //! Tests module for the [`layout`] module.
    //!
    //! [`layout`]: super

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
            for (text, source_range, cell_count) in [("👨‍👩‍👧‍👦", 11..18, 2), ("    \u{301}", 18..21, 4), ("x", 21..22, 1)]
            {
                let row = text_layout.next_row().expect("pending or subsequent row");

                assert_eq!(row.text(), text);
                assert_eq!(row.source_range(), source_range);
                assert_eq!(row.cells().len(), cell_count);
            }

            assert!(text_layout.next_row().is_none());
            assert!(text_layout.next_row().is_none(), "finished traversal stays exhausted");
        }
    }
}
