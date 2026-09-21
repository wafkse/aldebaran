//! Common capability bundle for fully featured source representations.
//!
//! [`Standard`] names the complete source-processing contract used by generic
//! text and rendering code. It combines iteration, dissection, line handling,
//! metadata, comparison, hashing, and owned conversion without adding new state.

use crate::{
    diff::SourceDiff, dissect::SourceDissect, hash::SourceHash, iter::SourceIter, line::SourceLines, metadata::SourceMetadata,
    owned::SourceOwned, source::Source,
};

/// Source supporting the ordinary Aldebaran source-processing operations.
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
        let _ = source;
    }

    #[test]
    fn strings_and_bytes_are_standard() {
        require_standard("source text");
        require_standard(b"source bytes".as_slice());
    }
}
