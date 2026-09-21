//! Common interface for handles spanning one or more source lines.
//!
//! A line extent exposes cheap forward iteration and aggregate metadata without
//! requiring random access to every line. Concrete handles can therefore retain
//! source-specific iteration state while renderers consume one uniform interface.

use crate::line::{LineId, Metadata, SourceLines};

/// A trait for types that can be used as a handle over one or more lines in a
/// specific [`source`](crate::source::Source).
///
/// # Remarks
///
/// Implementors of this trait are expected to provide a cheap way to iterate
/// over the lines in the source. In other words, this must serve as a cheap
/// handle to be used for iteration, not for random access.
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
