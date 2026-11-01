//! Hashing support for source storage keys.
//!
//! See [`SourceHash`] for further information.

use core::hash::Hash;

use crate::source::Source;

/// A trait for those [`Source`] types that can be hashed.
///
/// Sources that compare equal for storage must also have equal hashes.
pub trait SourceHash<'a>: Source<'a>
where
    Self: Hash,
{
}

/// Blanket implementation for every [`Source`] that is also [`Hash`].
impl<'a, S> SourceHash<'a> for S where S: Source<'a> + Hash + ?Sized {}
