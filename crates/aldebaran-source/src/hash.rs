//! Hashing support for source storage keys.
//!
//! [`SourceHash`] supplies the standard [`Hash`] capability alongside [`Source`].
//! When storage uses [`SourceDiff`](crate::diff::SourceDiff) to compare keys,
//! its content equivalence must agree with their hashes.

use core::hash::Hash;

use crate::source::Source;

/// Source keys whose hash can select a storage bucket.
/// Content equality used by a storage consumer must imply equal hashes.
pub trait SourceHash<'a>: Source<'a>
where
    Self: Hash,
{
}

/// Blanket implementation for every [`Source`] that is also [`Hash`].
impl<'a, S> SourceHash<'a> for S where S: Source<'a> + Hash + ?Sized {}
