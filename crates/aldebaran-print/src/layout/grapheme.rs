//! Identifies grapheme boundaries without tying consumption to row placement.
//!
//! [`GraphemeTraversal`] retains segmentation context for a logical line. Its
//! lookahead can identify a grapheme that does not fit the current row, while
//! the consumed offset keeps that grapheme available for the following row.

use core::ops::Range;

use unicode_segmentation::{GraphemeCursor, GraphemeIncomplete};

/// Traverses graphemes while retaining a boundary ahead of accepted input.
///
/// Repeated peeks return the pending byte range until the layout advances
/// `input_offset` to that range's end. Wrapping therefore preserves both the
/// pending grapheme and the segmentation context of preceding input.
#[derive(Debug, Clone)]
// NOTE(invariant): InputOffset <= Cursor.Position <= InputLength
pub(in crate::layout) struct GraphemeTraversal {
    /// Segmentation context, potentially ahead of the consumed input.
    cursor: GraphemeCursor,

    /// First visualization byte not yet accepted by a row.
    ///
    /// The layout advances this to the end of a range returned by `peek` only
    /// after the row accepts that grapheme.
    pub(in crate::layout) input_offset: usize,
}

impl GraphemeTraversal {
    /// Creates traversal for a visualization of `input_length` UTF-8 bytes.
    #[inline]
    pub(in crate::layout) fn new(input_length: usize) -> Self {
        let cursor = GraphemeCursor::new(usize::MIN, input_length, true);

        Self {
            cursor,
            input_offset: usize::MIN,
        }
    }

    /// Returns the next grapheme's visualization byte range without consuming it.
    ///
    /// `input` must be the unchanged visualization whose byte length was
    /// supplied at construction. `Ok(None)` indicates that no boundary remains
    /// after the consumed offset.
    ///
    /// # Errors
    ///
    /// Returns [`GraphemeIncomplete`] when the supplied input cannot satisfy
    /// the segmentation cursor's boundary or context request.
    #[inline]
    pub(in crate::layout) fn peek(&mut self, input: &str) -> Result<Option<Range<usize>>, GraphemeIncomplete> {
        let Self { cursor, input_offset } = self;

        let end = if cursor.cur_cursor() > *input_offset {
            Some(cursor.cur_cursor())
        } else {
            cursor.next_boundary(input, usize::MIN)?
        };

        Ok(end.map(|end| *input_offset..end))
    }
}
