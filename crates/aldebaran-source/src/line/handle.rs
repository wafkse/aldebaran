//! Repeatable traversal of a selected source-line range.
//!
//! [`LineHandle`] retains the original source and the terminator state at the
//! selection's beginning. Its [`LineExtent`] traversal therefore preserves
//! absolute coordinates and line numbers while stopping at the selected end.

use crate::dissect::SourceDissect;

use core::{fmt, hash, iter::Take};

use aldebaran_baseline::prelude::Spanned;

use crate::{
    iter::SourceIter,
    line::{LineExtent, LineId, LineIter, Metadata, SourceLines},
    terminate::Terminator,
};

/// A borrowed, nonempty line selection that can be traversed repeatedly.
///
/// Each traversal starts with the saved terminator state and retains the
/// original line numbers. The selection's metadata bounds that traversal, so
/// lines after the selection are not yielded.
// NOTE(invariant): The caller-supplied metadata, starting line, and terminator state must describe the same source selection.
pub struct LineHandle<'a, S, T>
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + SourceDissect<'a, View<'a> = &'a S> + ?Sized,
    T: Terminator<'a, S> + 'a,
{
    /// The original source backing the absolute line spans.
    source: &'a S,

    /// Aggregate bounds and line-length statistics computed for the selection.
    metadata: Metadata,

    /// First selected line, fixing the traversal's absolute offset and number.
    start_id: LineId<'a, S>,

    /// Termination policy and state immediately before the first selected line.
    terminator: T,
}

impl<'a, S, T> LineHandle<'a, S, T>
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + SourceDissect<'a, View<'a> = &'a S> + ?Sized,
    T: Terminator<'a, S> + 'a,
{
    /// Create a handle from a source, metadata, starting line, and terminator.
    ///
    /// `metadata` and `start_id` must describe the same nonempty contiguous
    /// selection in `source`, including its final line number, source-coordinate
    /// bounds, and length extrema. `terminator` must carry the state immediately
    /// before `start_id`.
    #[inline]
    pub const fn from_raw_parts(source: &'a S, metadata: Metadata, start_id: LineId<'a, S>, terminator: T) -> Self {
        Self {
            source,
            metadata,
            start_id,
            terminator,
        }
    }
}

impl<'a, S, T> LineExtent<'a, S> for LineHandle<'a, S, T>
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + SourceDissect<'a, View<'a> = &'a S> + ?Sized,
    T: Terminator<'a, S> + 'a,
{
    type Iterator = Take<LineIter<'a, S, T>>;

    #[inline]
    fn lines(&self) -> Self::Iterator {
        let &Self {
            source,
            ref start_id,
            terminator,
            ref metadata,
        } = self;

        let mut lines = LineIter::new_at_with(source, start_id.span().start(), terminator);

        lines.target_index = start_id.number();

        let count = metadata.lines().end().get() - start_id.number().get() + 1;

        lines.take(count)
    }

    #[inline]
    fn metadata(&self) -> &Metadata {
        let &Self { ref metadata, .. } = self;

        metadata
    }
}

impl<'a, S, T> LineHandle<'a, S, T>
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + SourceDissect<'a, View<'a> = &'a S> + ?Sized,
    T: Terminator<'a, S> + 'a,
{
    /// Original source against which all selected spans are interpreted.
    #[inline]
    pub const fn source(&self) -> &'a S {
        let &Self { source, .. } = self;

        source
    }

    /// Selection bounds and line-length extrema, including terminators.
    #[inline]
    pub const fn metadata(&self) -> &Metadata {
        let &Self { ref metadata, .. } = self;

        metadata
    }

    /// First selected line with its original source number and absolute extent.
    #[inline]
    pub const fn start(&self) -> &LineId<'a, S> {
        let Self { start_id, .. } = self;

        start_id
    }
}

impl<'a, S, T> hash::Hash for LineHandle<'a, S, T>
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + SourceDissect<'a, View<'a> = &'a S> + ?Sized,
    T: Terminator<'a, S> + 'a,
    S: hash::Hash,
    S::Line: hash::Hash,
    T: hash::Hash,
{
    #[inline]
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
        let Self {
            source,
            metadata,
            start_id: line_start_id,
            terminator,
        } = self;

        source.hash(state);
        metadata.hash(state);
        line_start_id.hash(state);
        terminator.hash(state);
    }
}

impl<'a, S, T> Eq for LineHandle<'a, S, T>
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + SourceDissect<'a, View<'a> = &'a S> + ?Sized,
    T: Terminator<'a, S> + 'a,
    S: Eq,
    S::Line: Eq,
    T: Eq,
{
}

impl<'a, S, T> PartialEq for LineHandle<'a, S, T>
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + SourceDissect<'a, View<'a> = &'a S> + ?Sized,
    T: Terminator<'a, S> + 'a,
    S: PartialEq,
    S::Line: PartialEq,
    T: Eq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let &Self {
            source,
            ref metadata,
            start_id: ref line_start_id,
            ref terminator,
        } = self;

        let &Self {
            source: other_source,
            metadata: ref other_metadata,
            start_id: ref other_line_start_id,
            terminator: ref other_terminator,
        } = other;

        source == other_source && metadata == other_metadata && line_start_id == other_line_start_id && terminator == other_terminator
    }
}

impl<'a, S, T> Clone for LineHandle<'a, S, T>
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + SourceDissect<'a, View<'a> = &'a S>,
    T: Terminator<'a, S> + 'a,
    S: Clone,
    S::Line: Clone,
    T: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let &Self {
            source,
            ref metadata,
            start_id: ref line_start_id,
            ref terminator,
        } = self;

        let metadata = metadata.clone();
        let line_start_id = *line_start_id;
        let terminator = *terminator;

        Self {
            source,
            metadata,
            start_id: line_start_id,
            terminator,
        }
    }
}

impl<'a, S, T> fmt::Debug for LineHandle<'a, S, T>
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + SourceDissect<'a, View<'a> = &'a S> + ?Sized,
    T: Terminator<'a, S> + 'a,
    S: fmt::Debug,
    S::Line: fmt::Debug,
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let &Self {
            ref source,
            ref metadata,
            start_id: ref line_start_id,
            ref terminator,
        } = self;

        f.debug_struct("LineExtents")
            .field("source", source)
            .field("metadata", metadata)
            .field("line_start_id", line_start_id)
            .field("terminator", &terminator)
            .finish()
    }
}
