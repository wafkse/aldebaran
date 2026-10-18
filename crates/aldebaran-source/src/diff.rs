//! Equality-oriented comparison between complete source values.
//!
//! [`SourceDiff`] provides the source-level comparison used by diff generation
//! without requiring every source representation to expose ordinary `Eq` as its
//! architectural contract. References forward comparison to the underlying source.

use crate::source::Source;

/// A trait for sources that can be differentiated.
///
/// This trait is vital for diff-generation.
pub trait SourceDiff<'a>: Source<'a> {
    /// Determine if the source is the same as another source.
    ///
    /// This is analogous to the [`Eq`] operator.
    fn same_as(&self, other: &Self) -> bool;
}

impl<'a, S> SourceDiff<'a> for &'a S
where
    S: SourceDiff<'a>,
{
    #[inline]
    fn same_as(&self, other: &Self) -> bool {
        S::same_as(*self, *other)
    }
}
