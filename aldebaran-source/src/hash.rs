//! Hashing facilities for source code.
//!
//! See the [`SourceHash`] trait for more information.

use core::hash::Hash;

use crate::source::Source;

/// A trait for those [`Source`] types that can be hashed.
pub trait SourceHash<'a>: Source<'a>
where
    Self: Hash,
{
}

/// Blanket implementation for all [`Source`] types that are also [`Hash`].
impl<'a, S> SourceHash<'a> for S where S: Source<'a> + Hash + ?Sized {}
