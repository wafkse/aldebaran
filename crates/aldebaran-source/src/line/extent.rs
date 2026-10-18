//! Traversal and metadata describing the same source-line selection.
//!
//! [`LineExtent`] connects a source-order sequence of [`LineId`] values with
//! the bounds and length extrema in [`Metadata`]. Consumers can size output
//! from the metadata before rendering the selected lines.

use crate::line::{LineId, Metadata, SourceLines};

/// A source-order line selection with matching aggregate bounds.
///
/// Implementations must keep the yielded line identities consistent with
/// their metadata, including original line numbers and terminator extents.
pub trait LineExtent<'a, S>
where
    S: SourceLines<'a> + ?Sized,
{
    /// Source-order traversal of the selected line identities.
    type Iterator: Iterator<Item = LineId<'a, S>>;

    /// Selected lines with their original numbers and absolute source spans.
    fn lines(&self) -> Self::Iterator;

    /// Bounds and line-length extrema covering exactly the yielded selection.
    fn metadata(&self) -> &Metadata;
}
