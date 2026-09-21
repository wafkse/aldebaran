//! Owned collections that guarantee at least one element.
//!
//! [`OneOrMore`] stores one mandatory first value separately from an owned tail.
//! Borrowed views and iterators preserve that nonempty guarantee while exposing
//! ordinary traversal and mutation over the complete logical collection.

mod iter;
mod reference;

pub use self::reference::RefOneOrMore;

use core::{convert::AsRef, num::NonZero};

use alloc::vec::Vec;
use iter::{OnceOrMore, OnceOrMoreMut};

/// A structure that guarantees that there is at least one known element.
///
/// For a reference-based version, see [`RefOneOrMore`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OneOrMore<T> {
    a0: T,
    a_n: Vec<T>,
}

impl<T> OneOrMore<T> {
    /// Create a [`OneOrMore`] of references to some type.
    #[inline]
    pub fn references<'a>((a0, a_n): (&'a T, &'a [T])) -> OneOrMore<&'a T> {
        let a0 = a0;
        let a_n = Vec::with_capacity(a_n.len());

        OneOrMore { a0, a_n }
    }

    /// Retrieve a reference to the first value contained in this [`OneOrMore`].
    #[inline]
    pub const fn first(&self) -> &T {
        let &Self { ref a0, .. } = self;

        a0
    }

    /// Retrieve a mutable reference to the first value contained in this
    /// [`OneOrMore`].
    #[inline]
    pub const fn first_mut(&mut self) -> &mut T {
        let &mut Self { ref mut a0, .. } = self;

        a0
    }

    /// Retrieve a reference to the [`Vec`] corresponding to the rest of the
    /// values.
    #[inline]
    pub const fn rest(&self) -> &Vec<T> {
        let &Self { ref a_n, .. } = self;

        a_n
    }

    /// Retrieve a mutable reference to [`Vec`] corresponding to the rest of the
    /// values.
    #[inline]
    pub const fn rest_mut(&mut self) -> &mut Vec<T> {
        let &mut Self { ref mut a_n, .. } = self;

        a_n
    }

    /// Retrieve the first value and the rest of the values as a 2-tuple.
    #[inline]
    pub const fn tuple(&self) -> (&T, &Vec<T>) {
        let &Self { ref a0, ref a_n } = self;

        (a0, a_n)
    }

    /// Retrieve the first value and the rest of the values as a 2-tuple,
    /// mutably.
    #[inline]
    pub fn tuple_mut(&mut self) -> (&mut T, &mut Vec<T>) {
        let &mut Self { ref mut a0, ref mut a_n } = self;

        (a0, a_n)
    }

    /// Retrieve the `nth` value.
    ///
    /// `n` is constrained to be a non-zero value, valid indices are `1..=len`
    /// and will only fetch from the rest of the values, not the first one.
    ///
    /// For natural zero-based traversal across the complete collection, use
    /// [`Self::iter`] instead.
    #[inline]
    pub fn nth(&self, n: NonZero<usize>) -> Option<&T> {
        let &Self { ref a_n, .. } = self;

        let target_index = n.get() - 1; // REMARK: will never underflow, as `NonZero` is always at least 1.

        a_n.get(target_index)
    }

    /// Retrieve an iterator over all values.
    #[inline]
    pub fn iter(&self) -> OnceOrMore<'_, T> {
        let &Self { ref a0, ref a_n } = self;

        let a0 = RefOneOrMore { a0, a_n };

        a0.iter()
    }

    /// Retrieve an iterator over all values, mutably.
    #[inline]
    pub fn iter_mut(&mut self) -> OnceOrMoreMut<'_, T> {
        OnceOrMoreMut::from_mut_ref(self)
    }

    /// Convert this [`OneOrMore`] into a referential counterpart
    /// [`RefOneOrMore`].
    #[inline]
    pub fn as_ref(&self) -> RefOneOrMore<'_, T> {
        let &Self { ref a0, ref a_n } = self;

        RefOneOrMore { a0, a_n }
    }
}

impl<T> OneOrMore<T> {
    /// Create a new [`OneOrMore`] with a single value.
    #[inline]
    pub const fn single(a0: T) -> Self {
        let a_n = Vec::new();

        Self { a0, a_n }
    }

    /// Create a new [`OneOrMore`] from borrowed values.
    ///
    /// This will clone the values into the internal storage.
    #[inline]
    pub fn from_borrowed<P, U>(a0: P, a_n: U) -> Self
    where
        T: Clone,
        P: AsRef<T>,
        U: AsRef<[T]>,
    {
        let a0 = a0.as_ref().clone();

        // FIXME: make use of allocator_api when it's available, otherwise we
        // just manually implement it. let a_n = a_n.as_ref().
        // to_vec_in(A::default());

        let a_n = {
            let target_slice = a_n.as_ref();

            let mut a_n = Vec::with_capacity(target_slice.len());

            a_n.extend_from_slice(target_slice);

            a_n
        };

        Self { a0, a_n }
    }

    /// Create a new [`OneOrMore`] with the specified first value and the rest
    #[inline]
    pub fn from_raw_parts((a0, a_n): (T, Vec<T>)) -> Self {
        Self { a0, a_n }
    }

    /// Attempt to create a new [`OneOrMore`] collection from a [`Vec`].
    ///
    /// Returns `None` if the vector is empty.
    ///
    /// # Note
    ///
    /// This performs a [`Vec::swap_remove`], so the order of the elements in
    /// the is not kept in any way or shape.
    #[inline]
    pub fn from_vec(mut vec: Vec<T>) -> Option<Self> {
        match vec.first() {
            Some(_) => Some(Self {
                a0: vec.swap_remove(0),
                a_n: vec,
            }),
            None => None,
        }
    }
}
