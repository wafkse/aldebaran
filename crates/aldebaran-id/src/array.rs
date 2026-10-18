//! Positional groups of identifiers with a fixed cardinality.
//!
//! [`IdArray`] groups IDs by position and permits duplicates, with
//! named slot accessors available for lengths 1 through 32.

use core::{
    fmt,
    ops::{Deref, DerefMut},
};

use aldebaran_baseline::tuple::Uniform;

use crate::prelude::Id;

/// A positional group of IDs whose length is part of its type.
/// Slots remain independently replaceable and need not contain distinct IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// NOTE(invariant): None.
pub struct IdArray<I: Id, const N: usize>([I; N]);

impl<I: Id, const N: usize> IdArray<I, N> {
    /// Preserve the supplied slot assignments, including repeated IDs.
    #[inline]
    pub const fn of(storage: [I; N]) -> Self {
        Self(storage)
    }

    /// Convert a homogeneous tuple into an array, preserving element order.
    #[inline]
    pub fn uniform<A>(value: A) -> Self
    where
        A: Uniform<N, I>,
    {
        Self::of(value.array())
    }
}

impl<I: Id, const N: usize> IntoIterator for IdArray<I, N> {
    type IntoIter = <[I; N] as IntoIterator>::IntoIter;
    type Item = I;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self(storage) = self;

        storage.into_iter()
    }
}

impl<I: Id, const N: usize> fmt::Display for IdArray<I, N>
where
    I: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let &Self(ref storage) = self;

        write!(f, "{:?}", storage)
    }
}

impl<I: Id, const N: usize> Deref for IdArray<I, N> {
    type Target = [I; N];

    #[inline]
    fn deref(&self) -> &Self::Target {
        let &Self(ref storage) = self;

        storage
    }
}

impl<I: Id, const N: usize> DerefMut for IdArray<I, N> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        let &mut Self(ref mut storage) = self;

        storage
    }
}

/// Named slots available only for array lengths that contain them.
macro_rules! Accessors {
    (
        $target_start:literal .. $target_end:literal for $target_ty:ident
    ) => {
        tokel::stream! {
            Accessors! {
                for $target_ty

                [< >]:sequence[[ $target_start..=$target_end ]]:reverse
            }
        }
    };
    (
        for $target_ty:ident

        $target_length:literal

        $(
            $(
                $target_index:literal
            )+
        )?
    ) => {
        tokel::stream! {
            $(
                impl<I: Id> $target_ty <I, $target_length> {
                    $(
                        #[doc = concat!("Zero-based slot ", stringify!($target_index), ", whose availability is guaranteed by this array length.")]
                        #[inline]
                        #[must_use = "redundant access to identifier"]
                        pub const fn [< id $target_index >]:to_string:flatten:concatenate:unstringify (&self) -> &I {
                            let &Self(ref storage) = self;

                            &storage[$target_index]
                        }

                        #[doc = concat!("Edit zero-based slot ", stringify!($target_index), " without changing the length or other slots. Duplicate IDs remain permitted.")]
                        #[inline]
                        #[must_use = "redundant mutable access to identifier"]
                        pub const fn [< id $target_index _mut >]:to_string:flatten:concatenate:unstringify (&mut self) -> &mut I {
                            let &mut Self(ref mut storage) = self;

                            &mut storage[$target_index]
                        }

                        #[doc = concat!("Project from zero-based slot ", stringify!($target_index), " without requiring the caller to copy its ID. The slot exists for this array length.")]
                        #[inline]
                        #[must_use = "redundant access to identifier"]
                        pub fn [< id $target_index _with >]:to_string:flatten:concatenate:unstringify <F, T> (&self, target_closure: F) -> T
                        where
                            F: FnOnce(&I) -> T
                        {
                            let &Self(ref storage) = self;

                            target_closure(&storage[$target_index])
                        }

                        #[doc = concat!("Edit zero-based slot ", stringify!($target_index), " through a closure while preserving the length and other slots. Duplicate IDs remain permitted.")]
                        #[inline]
                        #[must_use = "redundant access to identifier"]
                        pub fn [< id $target_index _mut_with >]:to_string:flatten:concatenate:unstringify <F, T> (&mut self, target_closure: F) -> T
                        where
                            F: FnOnce(&mut I) -> T
                        {
                            let &mut Self(ref mut storage) = self;

                            target_closure(&mut storage[$target_index])
                        }
                    )+
                }
            )?
        }

        $(
            Accessors! {
                for $target_ty

                $(
                    $target_index
                )+
            }
        )?
    };
    () => {};
}

Accessors!(0..32 for IdArray);
