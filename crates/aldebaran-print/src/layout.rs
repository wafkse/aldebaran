//! Lays out source visualizations as rows of terminal cells with source provenance.
//!
//! [`SourceText`] associates displayed input with its original coordinates.
//! Borrowed strings use UTF-8 byte offsets, while component visualizations can
//! retain a different source unit. [`TextLayout`] wraps that input at grapheme
//! boundaries and lends [`TextRow`] views whose [`Cell`] records connect terminal
//! columns to the original source.
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
/// its extent in the caller's source units. Graphemes may span component writes,
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
    /// Returns the previous input alongside the new layout. The caller can
    /// recover its owned capacity through [`SourceText::cleared`], while the new
    /// layout carries only the new input's lifetime. Segmentation, source
    /// projection, and row position all restart for the replacement input.
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
mod tests;
