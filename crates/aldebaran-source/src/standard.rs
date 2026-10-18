//! Shared capability bound for parsing, reporting, and source storage.
//!
//! [`Standard`] combines traversal and line access with content comparison,
//! hashing, metadata, and owned conversion. Generic consumers can require this
//! set while narrower operations continue to use the individual traits.

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

#[cfg(test)]
mod tests {
    use super::Standard;

    fn require_standard<'source, S>(source: &'source S)
    where
        S: Standard<'source> + ?Sized,
    {
        let _: &'source S = source;
    }

    #[test]
    fn strings_and_bytes_are_standard() {
        require_standard("source text");
        require_standard(b"source bytes".as_slice());
    }
}
