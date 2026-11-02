//! Sets of Aldebaran identifier values.
//!
//! See [`IdSet`] for further information.

use core::{
    hash::{BuildHasher, BuildHasherDefault, Hash},
    ops::Deref,
};

use crate::prelude::{Id, IdMap};

use rustc_hash::FxHasher;

/// A set of identifiers.
///
/// This is a set of identifiers, where each identifier is unique. The values are
/// exposed as a slice whose order may change when an identifier is removed.
/// This type currently has no public constructor or conversion.
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
// NOTE(invariant): None.
pub struct IdSet<I, S = BuildHasherDefault<FxHasher>>(IdMap<I, I, S>)
where
    I: Id,
    S: BuildHasher;

impl<I, S> IdSet<I, S>
where
    I: Id,
    S: BuildHasher,
{
    /// Determine whether the target [`trait@Id`] is in this set.
    ///
    /// # Time Complexity
    ///
    /// This operation has the same expected `O(1)` time complexity as
    /// [`IdMap::contains_key`].
    #[inline]
    pub fn has<Q>(&self, id: Q) -> bool
    where
        Q: Eq + Hash,
        Q: AsRef<I>,
    {
        let &Self(ref map, ..) = self;

        map.contains_key(id)
    }

    /// Insert the target [`trait@Id`] into this set, if it is not already present.
    ///
    /// If the target [`trait@Id`] is already present, this operation returns it
    /// to the caller.
    ///
    /// # Time Complexity
    ///
    /// This operation has the same expected, amortized `O(1)` time complexity as
    /// [`IdMap::insert`].
    #[inline]
    pub fn insert(&mut self, id: I) -> Option<I> {
        let &mut Self(ref mut map, ..) = self;

        map.insert(id, id)
    }

    /// Remove the target [`trait@Id`] from this set, if present.
    ///
    /// This returns the removed identifier, or [`None`] if it was absent.
    ///
    /// # Time Complexity
    ///
    /// This operation has the same expected, amortized `O(1)` time complexity as
    /// [`IdMap::remove`].
    #[inline]
    pub fn remove<Q>(&mut self, id: Q) -> Option<I>
    where
        Q: AsRef<I>,
        Q: Eq + Hash,
    {
        let &mut Self(ref mut map, ..) = self;

        map.remove(id).map(|(id, _)| id)
    }

    /// Return a slice containing all the [`trait@Id`]s in this set.
    ///
    /// # Time Complexity
    ///
    /// This operation is `O(1)`. The slice order may change after removal.
    #[inline]
    pub fn as_slice(&self) -> &[I] {
        let &Self(ref map, ..) = self;

        map.as_values()
    }
}

impl<I, S> Deref for IdSet<I, S>
where
    I: Id,
    S: BuildHasher,
{
    type Target = [I];

    #[inline]
    fn deref(&self) -> &Self::Target {
        let &Self(ref map, ..) = self;

        map.as_values()
    }
}

impl<I, S> Extend<I> for IdSet<I, S>
where
    I: Id,
    S: BuildHasher,
{
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = I>,
    {
        let &mut Self(ref mut map, ..) = self;

        map.extend(iter.into_iter().map(|id| (id, id)));
    }
}
