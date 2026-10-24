//! Relates grapheme byte ranges to the original source units they represent.
//!
//! [`SourceProjection`] supports forward traversal through a [`SourceText`].
//! Borrowed UTF-8 input has a direct offset relation, while component input can
//! require fragment records whose source widths differ from visualization lengths.

use core::ops::Range;

use super::SourceText;

/// Projects visualization ranges while retaining progress through source fragments.
///
/// Requests belong to the same input and must have nondecreasing starting byte
/// offsets. A repeated request is permitted, which lets a pending grapheme keep
/// its source extent when wrapping defers it to the next row.
#[derive(Debug, Clone, Default)]
// NOTE(invariant): FragmentIndex Never Decreases Within CurrentInput
pub(in crate::layout) struct SourceProjection {
    /// Earliest fragment still eligible to contribute to a projected range.
    fragment_index: usize,
}

impl SourceProjection {
    /// Returns the original source extent represented by a grapheme's bytes.
    ///
    /// `text_byte_range` must describe a nonempty grapheme within `input`.
    /// Component input contributes the source extents of the first through last
    /// intersecting fragments, so a projection can cover more than the selected
    /// visualization bytes. Returns `None` if no source fragment remains at the
    /// requested position. Direct UTF-8 projection requires no fragment lookup.
    pub(in crate::layout) fn project(&mut self, input: &SourceText<'_>, text_byte_range: Range<usize>) -> Option<Range<usize>> {
        let Self { fragment_index } = self;

        let SourceText {
            source_fragment_list,
            source_start,
            ..
        } = input;

        if source_fragment_list.is_empty() {
            Some(*source_start + text_byte_range.start..*source_start + text_byte_range.end)
        } else {
            while source_fragment_list
                .get(*fragment_index)
                .is_some_and(|fragment| fragment.text_byte_range.end <= text_byte_range.start)
            {
                *fragment_index += 1;
            }

            let first = source_fragment_list.get(*fragment_index)?;
            let mut last = *fragment_index;

            while source_fragment_list
                .get(last + 1)
                .is_some_and(|fragment| fragment.text_byte_range.start < text_byte_range.end)
            {
                last += 1;
            }

            let last = source_fragment_list.get(last)?;

            Some(first.source_range.start..last.source_range.end)
        }
    }
}
