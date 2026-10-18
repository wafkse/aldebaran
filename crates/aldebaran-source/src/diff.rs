//! Content comparison between source values.
//!
//! [`SourceDiff`] lets consumers define content identity independently of the
//! source representation's [`Eq`] implementation.

use crate::source::Source;

/// Compare two source values under the implementation's content identity.
pub trait SourceDiff<'a>: Source<'a> {
    /// Return whether both sources represent the same value.
    ///
    /// Implementations should define an equivalence relation. If the source
    /// also implements [`crate::hash::SourceHash`], equal values must hash equally.
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
