//! Membership and ordered views of distinct typed IDs.
//!
//! [`IdSet`] uses ID equality for membership and permits removal to reorder its
//! slice view, so positions do not serve as stable identities.

use core::{
    hash::{BuildHasher, BuildHasherDefault, Hash},
    ops::Deref,
};

use crate::prelude::{Id, IdMap};

use rustc_hash::FxHasher;

/// Retain distinct IDs with a slice view whose order may change on removal.
/// No public constructor or conversion currently permits creating a set.
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
    /// Membership under ID equality, independent of slice position.
    #[inline]
    pub fn has<Q>(&self, id: Q) -> bool
    where
        Q: Eq + Hash,
        Q: AsRef<I>,
    {
        let &Self(ref map, ..) = self;

        map.contains_key(id)
    }

    /// Insert an identifier, returning the existing value if already present.
    #[inline]
    pub fn insert(&mut self, id: I) -> Option<I> {
        let &mut Self(ref mut map, ..) = self;

        map.insert(id, id)
    }

    /// Remove and return an identifier if present.
    ///
    /// Removal may reorder the slice view.
    ///
    /// # Panics
    ///
    /// Panics after removing an identifier that occupies the final storage position.
    #[inline]
    pub fn remove<Q>(&mut self, id: Q) -> Option<I>
    where
        Q: AsRef<I>,
        Q: Eq + Hash,
    {
        let &mut Self(ref mut map, ..) = self;

        map.remove(id).map(|(id, _)| id)
    }

    /// Distinct IDs in their current storage order, which removal may change.
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
