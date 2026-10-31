//! Traversal and metadata describing the same source-line selection.
//!
//! [`LineExtent`] connects a source-order sequence of [`LineId`] values with
//! the bounds and length extrema in [`Metadata`]. Consumers can size output
//! from the metadata before rendering the selected lines.

use crate::line::{LineId, Metadata, SourceLines};

/// A trait for types that can be used as a handle over one or more lines in a
/// specific [`source`](crate::source::Source).
///
/// # Remarks
///
/// Implementors of this trait are expected to provide a cheap way to iterate
/// over the lines in the source. In other words, this must serve as a cheap
/// handle to be used for iteration, not for random access.
///
/// Implementations must keep the yielded line identities consistent with
/// their metadata, including original line numbers and terminator extents.
pub trait LineExtent<'a, S>
where
    S: SourceLines<'a> + ?Sized,
{
    /// The associated [`Iterator`] type for this handle.
    ///
    /// See [`LineId`] for more information.
    type Iterator: Iterator<Item = LineId<'a, S>>;

    /// Provide an iterator over the lines relevant to this handle.
    fn lines(&self) -> Self::Iterator;

    /// Determine the associated [`metadata`](Metadata) for this handle.
    fn metadata(&self) -> &Metadata;
}
