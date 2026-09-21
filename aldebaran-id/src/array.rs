//! Fixed-size collections of identifiers.
//!
//! [`IdArray`] stores a compile-time number of identifiers and provides indexed
//! access while preserving the concrete identifier type. The collection is used
//! where identifier cardinality is part of the surrounding type contract.

use core::{
    fmt,
    ops::{Deref, DerefMut},
};

use aldebaran_dsa::appendage::Uniform;

use crate::prelude::Id;

/// An array of [`trait@Id`]'s of length `N`.
///
/// This exposes various methods for accessing each individual [`trait@Id`] in the
/// array, this is given as a sequence of `IdArray::i{{N}}`, where `N` is the
/// index of the [`trait@Id`] in the array plus one, i.e these methods follow the
/// trend `id1..id{{N}}`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IdArray<I: Id, const N: usize>([I; N]);

impl<I: Id, const N: usize> IdArray<I, N> {
    /// Create a new [`IdArray`] from the given storage array.
    #[inline]
    pub const fn of(storage: [I; N]) -> Self {
        Self(storage)
    }

    /// Create a new [`IdArray`] from the target [`Uniform`] appendage.
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

/// A macro that generates accessors for the [`IdArray`] type.
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
                        #[doc = concat!("Access the `", stringify!($target_index), "th` identifier `I` contained in this [`", stringify!($target_ty), "`].")]
                        #[inline]
                        #[must_use = "redundant access to identifier"]
                        pub const fn [< id $target_index >]:to_string:flatten:concatenate:unstringify (&self) -> &I {
                            let &Self(ref storage) = self;

                            &storage[$target_index]
                        }

                        #[doc = concat!("Mutably access the `", stringify!($target_index), "th` identifier `I` contained in this [`", stringify!($target_ty), "`].")]
                        #[inline]
                        #[must_use = "redundant mutable access to identifier"]
                        pub const fn [< id $target_index _mut >]:to_string:flatten:concatenate:unstringify (&mut self) -> &mut I {
                            let &mut Self(ref mut storage) = self;

                            &mut storage[$target_index]
                        }

                        #[doc = concat!("Access the `", stringify!($target_index), "th` identifier `I` contained in this [`", stringify!($target_ty), "`] through a closure.")]
                        #[inline]
                        #[must_use = "redundant access to identifier"]
                        pub fn [< id $target_index _with >]:to_string:flatten:concatenate:unstringify <F, T> (&self, target_closure: F) -> T
                        where
                            F: FnOnce(&I) -> T
                        {
                            let &Self(ref storage) = self;

                            target_closure(&storage[$target_index])
                        }

                        #[doc = concat!("Mutably access the `", stringify!($target_index), "th` identifier `I` contained in this [`", stringify!($target_ty), "`] through a closure.")]
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
