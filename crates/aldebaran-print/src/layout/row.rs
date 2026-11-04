//! Builds wrapped rows while retaining output capacity between advances.
//!
//! See [`RowBuffer`], [`RowBuilder`], and [`RowTraversal`] for further information.

use alloc::{string::String, vec::Vec};
use core::{iter, num::NonZero, ops::Range};

use super::{Cell, TextRow, terminal::TerminalGrapheme};

/// Cell reservation floor for a double-width glyph in a one-column row.
///
/// This avoids growing the initial cell buffer for that glyph without imposing
/// a maximum grapheme width. The reservation is also bounded by input length.
const WIDE_GLYPH_CELL_COUNT: usize = 2;

/// Owns the text and cell capacity reused by successive output rows.
///
/// Only rows requiring normalization use the owned text buffer. Cell records
/// remain in this storage even when displayed text borrows the input instead.
/// Lending a row prevents either buffer from being mutated until its borrow ends.
#[derive(Debug, Clone, Default)]
// NOTE(invariant): None.
pub(in crate::layout) struct RowBuffer {
    /// Owned text used only when terminal normalization is necessary.
    text: String,

    /// Cells for the most recently built row.
    cell_list: Vec<Cell>,
}

impl RowBuffer {
    /// Clears the previous row and reserves cells for a new line's initial row.
    ///
    /// Existing capacities are retained. The reservation uses the column budget
    /// and double-width glyph floor, bounded by `input_length`, so short input
    /// does not reserve storage for an arbitrarily large configured width.
    pub(in crate::layout) fn prepare(&mut self, input_length: usize, column_count: NonZero<usize>) {
        let Self { text, cell_list } = self;

        text.clear();
        cell_list.clear();

        let initial_cell_capacity = column_count.get().max(WIDE_GLYPH_CELL_COUNT).min(input_length);

        if cell_list.capacity() < initial_cell_capacity {
            cell_list.reserve_exact(initial_cell_capacity);
        }
    }

    /// Lends the prepared cells with their source extent and displayed text.
    ///
    /// `uses_normalized_text` selects owned normalized text instead of a slice of `input`.
    /// The supplied ranges must describe the row that produced these cells.
    /// Returns `None` if the borrowed-text range is outside the input or splits
    /// a UTF-8 scalar.
    fn lend<'row>(
        &'row self,
        input: &'row str,
        source_range: Range<usize>,
        input_byte_range: Range<usize>,
        uses_normalized_text: bool,
    ) -> Option<TextRow<'row>> {
        let Self { text, cell_list } = self;

        let text = if uses_normalized_text {
            text.as_str()
        } else {
            input.get(input_byte_range)?
        };

        Some(TextRow {
            source_range,
            text,
            cell_list,
        })
    }
}

/// Retains the source position and completion state of row traversal.
///
/// A wrapped continuation starts at the deferred grapheme's source coordinate.
/// Exhaustion is recorded after the final row is built, including the empty row
/// produced for empty input, so subsequent advances yield no further rows.
#[derive(Debug, Clone)]
// NOTE(invariant): None.
pub(in crate::layout) struct RowTraversal {
    /// Original coordinate at the next row's beginning.
    pub(in crate::layout) next_source_start: usize,

    /// Whether all input and its final source extent have been presented.
    pub(in crate::layout) is_exhausted: bool,
}

impl RowTraversal {
    /// Creates unfinished traversal at the input's initial source coordinate.
    #[inline]
    pub(in crate::layout) const fn new(next_source_start: usize) -> Self {
        Self {
            next_source_start,
            is_exhausted: false,
        }
    }
}

/// Assigns accepted graphemes to cells within a row's terminal budget.
///
/// The builder borrows reusable storage exclusively until it lends the finished
/// row. Its acceptance policy keeps the first visible unit intact even when it
/// exceeds the budget, and admits zero-width input without forcing a wrap.
///
/// Leading invisible input remains in the row's source extent. Later zero-width
/// input extends its preceding visible cell and wide continuations. Text byte
/// ranges stay relative to the displayed row, independently of source units.
// NOTE(invariant): ForAll(Cell In RowBuffer.CellList): Cell.SourceRange Within SourceRange
pub(in crate::layout) struct RowBuilder<'buffer> {
    /// Exclusive access to reusable row storage until the row is lent.
    row_buffer: &'buffer mut RowBuffer,

    /// Original source extent represented by accepted input.
    pub(in crate::layout) source_range: Range<usize>,

    /// Accepted visualization bytes, retained for the borrowed-text path.
    input_byte_range: Range<usize>,

    /// Budget permitting an indivisible first grapheme to exceed it.
    column_count: NonZero<usize>,

    /// Whether the text buffer contains the row's terminal-safe output.
    uses_normalized_text: bool,
}

impl<'buffer> RowBuilder<'buffer> {
    /// Starts an empty row at the supplied source and visualization positions.
    ///
    /// Previous output is discarded while its capacity is retained. Source
    /// coordinates and visualization byte offsets are tracked separately.
    #[inline]
    pub(in crate::layout) fn new(
        row_buffer: &'buffer mut RowBuffer,
        source_start: usize,
        input_start: usize,
        column_count: NonZero<usize>,
    ) -> Self {
        let RowBuffer { text, cell_list } = row_buffer;

        text.clear();
        cell_list.clear();

        let source_range = source_start..source_start;
        let input_byte_range = input_start..input_start;

        Self {
            row_buffer,
            source_range,
            input_byte_range,
            column_count,
            uses_normalized_text: false,
        }
    }

    /// Returns the terminal column immediately after the accepted cells.
    #[inline]
    pub(in crate::layout) const fn column(&self) -> usize {
        let Self { row_buffer, .. } = self;

        let RowBuffer { cell_list, .. } = row_buffer;

        cell_list.len()
    }

    /// Reports whether a grapheme of `width` can join the current row.
    ///
    /// Zero-width input and the first visible unit are accepted regardless of
    /// the budget. Later visible input must fit within the configured columns.
    #[inline]
    pub(in crate::layout) const fn accepts(&self, width: usize) -> bool {
        let Self {
            row_buffer, column_count, ..
        } = self;

        let RowBuffer { cell_list, .. } = row_buffer;

        let exceeds_column_budget = cell_list.len().saturating_add(width) > column_count.get();

        width == usize::MIN || cell_list.is_empty() || !exceeds_column_budget
    }

    /// Adds an accepted grapheme's displayed text and source provenance to the row.
    ///
    /// The width must be accepted for the current row. Its byte range must follow
    /// the row's accepted visualization bytes in `input`, and its terminal form
    /// must have been measured at the current row column.
    /// Wide text receives continuation cells, while tabs give each expanded
    /// space a text range. Neither operation changes the original source units.
    ///
    /// Returns `None` if switching to normalized output requires a prefix slice
    /// that is outside `input` or does not lie on UTF-8 boundaries.
    pub(in crate::layout) fn push(
        &mut self,
        input: &str,
        input_byte_range: Range<usize>,
        mut source_range: Range<usize>,
        terminal_grapheme: TerminalGrapheme<'_>,
    ) -> Option<()> {
        let Self {
            row_buffer,
            source_range: row_source_range,
            input_byte_range: row_input_byte_range,
            uses_normalized_text,
            ..
        } = self;

        let RowBuffer { text, cell_list } = row_buffer;

        if terminal_grapheme.transformed() && !*uses_normalized_text {
            text.push_str(input.get(row_input_byte_range.start..input_byte_range.start)?);
            *uses_normalized_text = true;
        }

        let cell_text_start = if *uses_normalized_text {
            text.len()
        } else {
            input_byte_range.start - row_input_byte_range.start
        };

        if *uses_normalized_text {
            terminal_grapheme.append(text);
        }

        let text_end = if *uses_normalized_text {
            text.len()
        } else {
            input_byte_range.end - row_input_byte_range.start
        };
        let width = terminal_grapheme.width();

        match terminal_grapheme {
            TerminalGrapheme::Tab { .. } => {
                cell_list.reserve(width);

                for tab_cell_index in usize::MIN..width {
                    let is_first_row_cell = cell_list.is_empty();
                    let source_range = if is_first_row_cell {
                        row_source_range.start..source_range.end
                    } else {
                        source_range.clone()
                    };
                    let cell_text_start_index = if is_first_row_cell {
                        usize::MIN
                    } else {
                        cell_text_start + tab_cell_index
                    };
                    let text_byte_range = cell_text_start_index..cell_text_start + tab_cell_index + 1;

                    cell_list.push(Cell {
                        source_range,
                        text_byte_range,
                    });
                }
            }
            TerminalGrapheme::Replacement { .. } | TerminalGrapheme::Verbatim { .. } => {
                if width == usize::MIN {
                    if let Some(leading_cell_index) = cell_list.iter().rposition(|cell| !cell.text_byte_range.is_empty()) {
                        for Cell {
                            source_range: cell_source_range,
                            ..
                        } in cell_list.iter_mut().skip(leading_cell_index)
                        {
                            cell_source_range.end = source_range.end;
                        }

                        if let Some(Cell { text_byte_range, .. }) = cell_list.get_mut(leading_cell_index) {
                            text_byte_range.end = text_end;
                        }
                    }
                } else {
                    if cell_list.is_empty() {
                        source_range.start = row_source_range.start;
                    }

                    let cell_text_start_index = if cell_list.is_empty() { usize::MIN } else { cell_text_start };
                    let text_byte_range = cell_text_start_index..text_end;
                    let cell = {
                        let source_range = source_range.clone();

                        Cell {
                            source_range,
                            text_byte_range,
                        }
                    };

                    cell_list.push(cell);

                    let source_range = source_range.clone();
                    let text_byte_range = text_end..text_end;
                    let continuation_cell = Cell {
                        source_range,
                        text_byte_range,
                    };

                    cell_list.extend(iter::repeat_n(continuation_cell, width - 1));
                }
            }
        }

        row_source_range.end = source_range.end;
        row_input_byte_range.end = input_byte_range.end;

        Some(())
    }

    /// Consumes the builder and lends its row from the retained output storage.
    ///
    /// `input` must be the visualization used while building the row. Normalized
    /// rows borrow owned output text, while unchanged rows borrow the accepted
    /// input bytes. Returns `None` if that input slice cannot be obtained.
    #[inline]
    pub(in crate::layout) fn finish(self, input: &'buffer str) -> Option<TextRow<'buffer>> {
        let Self {
            row_buffer,
            source_range,
            input_byte_range,
            uses_normalized_text,
            ..
        } = self;

        row_buffer.lend(input, source_range, input_byte_range, uses_normalized_text)
    }
}
