//! Sets of Aldebaran identifier values.
//!
//! [`IdSet`] tracks unique identifiers and exposes membership, insertion, removal,
//! and slice access over the stored values. The concrete identifier type remains
//! visible throughout the API rather than being normalized to a primitive integer.

use core::{
    hash::{BuildHasher, BuildHasherDefault, Hash},
    ops::Deref,
};

use crate::prelude::{Id, IdMap};

use aldebaran_hash::fxhash::FxHasher;

/// A set of identifiers.
///
/// This is a set of identifiers, where each identifier is unique.
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct IdSet<I, S = BuildHasherDefault<FxHasher>>
where
    I: Id,
    S: BuildHasher,
{
    map: IdMap<I, I, S>,
}

impl<I, S> IdSet<I, S>
where
    I: Id,
    S: BuildHasher,
{
    /// Determine whether if the target [`trait@Id`] is in this set.
    ///
    /// # Time Complexity
    ///
    /// This operation is `O(1)` (amortized), i.e the same as
    /// [`IdMap::contains_key`].
    #[inline]
    pub fn has<Q: AsRef<I>>(&self, id: Q) -> bool
    where
        Q: Eq + Hash,
    {
        let &Self { ref map, .. } = self;

        map.contains_key(id)
    }

    /// Insert the target [`trait@Id`] into this set, if it is not already present.
    ///
    /// In the event that the target [`trait@Id`] is already present, this operation
    /// will return the target [`trait@Id`] back to the caller.
    ///
    /// # Time Complexity
    ///
    /// This operation is `O(1)` (amortized), i.e the same as [`IdMap::insert`].
    #[inline]
    pub fn insert(&mut self, id: I) -> Option<I> {
        let &mut Self { ref mut map, .. } = self;

        map.insert(id, id)
    }

    /// Remove the target [`trait@Id`] from this set, if it is present.
    ///
    /// In the event that the target [`trait@Id`] is not present, this operation will
    /// return `None`.
    ///
    /// # Time Complexity
    ///
    /// This operation is `O(1)` (amortized), i.e the same as [`IdMap::remove`].
    #[inline]
    pub fn remove<Q>(&mut self, id: Q) -> Option<I>
    where
        Q: AsRef<I>,
        Q: Eq + Hash,
    {
        let &mut Self { ref mut map, .. } = self;

        map.remove(id).map(|(id, _)| id)
    }

    /// Fetch a slice to an array containing all the [`trait@Id`]s in this set.
    ///
    /// # Time Complexity
    ///
    /// This operation is `O(1)`.
    #[inline]
    pub fn as_slice(&self) -> &[I] {
        let &Self { ref map, .. } = self;

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
        let &Self { ref map, .. } = self;

        map.as_values()
    }
}

impl<I, S> Extend<I> for IdSet<I, S>
where
    I: Id,
    S: BuildHasher,
{
    fn extend<T: IntoIterator<Item = I>>(&mut self, iter: T) {
        let &mut Self { ref mut map, .. } = self;

        map.extend(iter.into_iter().map(|id| (id, id)));
    }
}
