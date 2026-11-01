//! Shared capability bound for parsing, reporting, and source storage.
//!
//! See [`Standard`] for further information.

use crate::{
    diff::SourceDiff, dissect::SourceDissect, hash::SourceHash, iter::SourceIter, line::SourceLines, metadata::SourceMetadata,
    owned::SourceOwned, source::Source,
};

/// Source capabilities needed across parsing, diagnostic rendering, and storage.
/// All sources satisfying the constituent contracts receive this bound automatically.
pub trait Standard<'source>:
    Source<'source>
    + SourceDissect<'source>
    + SourceIter<'source>
    + SourceMetadata<'source>
    + SourceLines<'source>
    + SourceDiff<'source>
    + SourceHash<'source>
    + SourceOwned<'source>
{
}

impl<'source, S> Standard<'source> for S where
    S: Source<'source>
        + SourceDissect<'source>
        + SourceIter<'source>
        + SourceMetadata<'source>
        + SourceLines<'source>
        + SourceDiff<'source>
        + SourceHash<'source>
        + SourceOwned<'source>
        + ?Sized
{
}
