//! Concrete source-backed handles over contiguous line extents.
//!
//! [`LineHandle`] retains the original source, starting line identity, terminator,
//! and aggregate metadata needed to recreate line iteration. Absolute source
//! coordinates remain valid because the handle never substitutes a relative slice.

use core::{fmt, hash};

use aldebaran_span::prelude::Spanned;

use crate::{
    iter::SourceIter,
    line::{LineExtent, LineId, LineIter, Metadata, SourceLines},
    terminate::Terminator,
};

/// The known extents of a collection of lines.
///
/// Used to query for the general information over a collection of lines, as
/// well as iterate over them.
pub struct LineHandle<'a, S, T>
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + crate::dissect::SourceDissect<'a, View<'a> = &'a S> + ?Sized,
    T: Terminator<'a, S> + 'a,
{
    source: &'a S,
    metadata: Metadata,
    start_id: LineId<'a, S>,
    terminator: T,
}

impl<'a, S, T> LineHandle<'a, S, T>
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + crate::dissect::SourceDissect<'a, View<'a> = &'a S> + ?Sized,
    T: Terminator<'a, S> + 'a,
{
    /// Create a new [`LineHandle`] from its raw parts: the source, the
    /// metadata, and the starting line identifier.
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
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + crate::dissect::SourceDissect<'a, View<'a> = &'a S> + ?Sized,
    T: Terminator<'a, S> + 'a,
{
    type Iterator = LineIter<'a, S, T>;

    #[inline]
    fn lines(&self) -> Self::Iterator {
        let &Self {
            source,
            ref start_id,
            terminator,
            ..
        } = self;

        let mut lines = LineIter::new_at_with(source, start_id.span().start(), terminator);
        lines.target_index = start_id.number();
        lines
    }

    #[inline]
    fn metadata(&self) -> &Metadata {
        let &Self { ref metadata, .. } = self;

        metadata
    }
}

impl<'a, S, T> LineHandle<'a, S, T>
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + crate::dissect::SourceDissect<'a, View<'a> = &'a S> + ?Sized,
    T: Terminator<'a, S> + 'a,
{
    /// Retrieve the [`source`](crate::source::Source) that this handle represents.
    #[inline]
    pub const fn source(&self) -> &'a S {
        let &Self { source, .. } = self;

        source
    }

    /// Retrieve the associated metadata that this handle represents.
    #[inline]
    pub const fn metadata(&self) -> &Metadata {
        let &Self { ref metadata, .. } = self;

        metadata
    }

    /// Retrieve the starting line identifier this handle represents.
    #[inline]
    pub const fn start(&self) -> &LineId<'a, S> {
        let Self { start_id, .. } = self;

        start_id
    }
}

impl<'a, S, T> hash::Hash for LineHandle<'a, S, T>
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + crate::dissect::SourceDissect<'a, View<'a> = &'a S> + ?Sized,
    T: Terminator<'a, S> + 'a,
    S: hash::Hash,
    S::Line: hash::Hash,
    T: hash::Hash,
{
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
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
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + crate::dissect::SourceDissect<'a, View<'a> = &'a S> + ?Sized,
    T: Terminator<'a, S> + 'a,
    S: Eq,
    S::Line: Eq,
    T: Eq,
{
}

impl<'a, S, T> PartialEq for LineHandle<'a, S, T>
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + crate::dissect::SourceDissect<'a, View<'a> = &'a S> + ?Sized,
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
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + crate::dissect::SourceDissect<'a, View<'a> = &'a S> + ?Sized,
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
        let line_start_id = line_start_id.clone();
        let terminator = terminator.clone();

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
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + crate::dissect::SourceDissect<'a, View<'a> = &'a S> + ?Sized,
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
