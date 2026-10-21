//! Collections with a required first value and an optional tail.
//!
//! [`Nonempty`] owns its values, while [`reference::RefNonempty`] shares a first
//! value and a slice without allocating. Both traverse the first value before
//! the tail, so diagnostic collections can retain a primary value separately
//! from related values.

pub mod reference;

use core::{
    iter::{self, FusedIterator},
    num::NonZero,
};

use alloc::vec::Vec;

use self::reference::RefNonempty;

/// An owned sequence whose required first value survives changes to its tail.
///
/// Iteration visits the first value followed by the tail in storage order, and
/// [`Self::as_ref`] shares that sequence without cloning its elements.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
// NOTE(invariant): None.
pub struct Nonempty<T>((T, Vec<T>));

impl<T> Nonempty<T> {
    /// Start a collection with an empty, growable tail.
    #[inline]
    pub const fn single(target_value: T) -> Self {
        Self((target_value, Vec::new()))
    }

    /// Place the required value before the supplied tail without reordering it.
    #[inline]
    pub const fn multiple(target_single: T, target_list: Vec<T>) -> Self {
        Self((target_single, target_list))
    }

    /// Accept an owned vector without changing its element order.
    ///
    /// An empty vector returns `None` because it cannot supply the required value.
    #[inline]
    pub fn from_vec(values: Vec<T>) -> Option<Self> {
        let mut values = values.into_iter();

        match values.next() {
            Some(first) => {
                let rest = values.collect();

                Some(Self::multiple(first, rest))
            }
            None => None,
        }
    }

    /// The required value and growable tail, in iteration order.
    #[inline]
    pub const fn tuple(&self) -> &(T, Vec<T>) {
        let Self(target_tuple) = self;

        target_tuple
    }

    /// Count the required value and tail, saturating at `usize::MAX`.
    #[inline]
    pub const fn len(&self) -> NonZero<usize> {
        let Self((.., target_list)) = self;

        NonZero::<usize>::MIN.saturating_add(target_list.len())
    }

    /// The value retained even when the tail has no elements.
    #[inline]
    pub const fn first(&self) -> &T {
        let Self((target_value, ..)) = self;

        target_value
    }

    /// Replace or modify the required value while keeping the collection nonempty.
    #[inline]
    pub const fn first_mut(&mut self) -> &mut T {
        let Self((target_value, ..)) = self;

        target_value
    }

    /// Elements following the required value, in iteration order.
    #[inline]
    pub const fn rest(&self) -> &[T] {
        let Self((.., target_list)) = self;

        target_list.as_slice()
    }

    /// Modify existing tail elements without changing the tail's length.
    #[inline]
    pub const fn rest_mut(&mut self) -> &mut [T] {
        let Self((.., target_list)) = self;

        target_list.as_mut_slice()
    }

    /// The growable tail, excluding the required value.
    #[inline]
    pub const fn list(&self) -> &Vec<T> {
        let Self((.., target_list)) = self;

        target_list
    }

    /// Insert or remove tail elements while the required value remains present.
    #[inline]
    pub const fn list_mut(&mut self) -> &mut Vec<T> {
        let Self((.., target_list)) = self;

        target_list
    }

    /// Visit the required value before the tail in storage order.
    #[inline]
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &T> + FusedIterator + use<'_, T> {
        let Self((target_value, target_list)) = self;

        iter::once(target_value).chain(target_list.iter())
    }

    /// Visit each value exclusively, starting with the required value.
    #[inline]
    pub fn iter_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut T> + FusedIterator + use<'_, T> {
        let Self((target_value, target_list)) = self;

        iter::once(target_value).chain(target_list.iter_mut())
    }

    /// Share the sequence with borrowed collection APIs without allocating.
    #[inline]
    pub const fn as_ref(&self) -> RefNonempty<'_, T> {
        let Self((first, rest)) = self;

        let rest = rest.as_slice();

        RefNonempty::multiple(first, rest)
    }
}

#[cfg(test)]
mod tests;
