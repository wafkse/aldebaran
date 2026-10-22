//! Nonzero integer identities with optional compile-time type distinctions.
//!
//! [`Id!`](macro@crate::Id) declares fixed-width types implementing [`trait@Id`], and [`TaggedId`]
//! associates an existing ID with a value type without proving storage membership.

use core::{
    cmp, fmt,
    hash::{self, Hash},
    marker::PhantomData,
};

use crate::zeroable::{NonZeroable, NonZeroed};

/// Declare a macro for fixed-width nonzero IDs, applying caller metadata to each type.
/// Optional display formatting uses the ID's primitive value.
#[macro_export]
macro_rules! Id {
    (
        $(
            #[$target_meta:meta]
        )*
        become $target_macro:ident

        $(
            as $target_fmt_str:literal
        )?
    ) => {
        #[doc = concat!("Declare fixed-width IDs with the supplied metadata. The `become` form also exposes the type under this macro's name.")]
        macro_rules! $target_macro {
            ($target_bits:literal) => {
                $crate::reexport::stream! {
                    #[doc = concat!("A ", stringify!($target_bits), "-bit identity that excludes zero without proving uniqueness or storage membership.")]
                    ///
                    #[must_use = "identifiers may not be lost"]
                    $(
                        #[$target_meta]
                    )*
                    pub struct [< $target_macro $target_bits >]:to_string:flatten:concatenate:unstringify(
                        pub(crate) $crate::zeroable::NonZeroed<[< u $target_bits >]:to_string:flatten:concatenate:unstringify>
                    );

                    impl From<[< $target_macro $target_bits >]:to_string:flatten:concatenate:unstringify>
                        for $crate::zeroable::NonZeroed<[< u $target_bits >]:to_string:flatten:concatenate:unstringify>
                    {
                        #[inline]
                        fn from(id: [< $target_macro $target_bits >]:to_string:flatten:concatenate:unstringify) -> Self {
                            let [< $target_macro $target_bits >]:to_string:flatten:concatenate:unstringify(inner) = id;

                            inner
                        }
                    }

                    impl AsRef<[< $target_macro $target_bits >]:to_string:flatten:concatenate:unstringify>
                        for [< $target_macro $target_bits >]:to_string:flatten:concatenate:unstringify
                    {
                        #[inline]
                        fn as_ref(&self) -> &Self {
                            self
                        }
                    }

                    $(
                        impl ::core::fmt::Display for [< $target_macro $target_bits >]:to_string:flatten:concatenate:unstringify {
                            #[inline]
                            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                                write!(f, $target_fmt_str, Into::<[< u $target_bits >]:to_string:flatten:concatenate:unstringify>::into(*self))
                            }
                        }
                    )?

                    impl From<[< $target_macro $target_bits >]:to_string:flatten:concatenate:unstringify>
                        for [< u $target_bits >]:to_string:flatten:concatenate:unstringify
                    {
                        #[inline]
                        fn from(id: [< $target_macro $target_bits >]:to_string:flatten:concatenate:unstringify) -> Self {
                            let [< $target_macro $target_bits >]:to_string:flatten:concatenate:unstringify(inner) = id;

                            inner.get()
                        }
                    }

                    impl $crate::prelude::Id for [< $target_macro $target_bits >]:to_string:flatten:concatenate:unstringify {
                        type BackingPrimitive = [< u $target_bits >]:to_string:flatten:concatenate:unstringify;

                        const MAX: Self = Self($crate::zeroable::NonZeroed::<[< u $target_bits >]:to_string:flatten:concatenate:unstringify>::MAX);
                        const MIN: Self = Self($crate::zeroable::NonZeroed::<[< u $target_bits >]:to_string:flatten:concatenate:unstringify>::MIN);

                        #[inline]
                        fn raw(value: $crate::zeroable::NonZeroed<Self::BackingPrimitive>) -> Self {
                            Self(value)
                        }

                        #[inline]
                        fn as_nonzeroed(&self) -> $crate::zeroable::NonZeroed<Self::BackingPrimitive> {
                            let &Self(inner) = self;

                            inner
                        }
                    }
                }
            };

            ($target_bits:literal become) => {
                $target_macro!($target_bits);

                $crate::reexport::stream! {
                    #[doc(inline)]
                    pub use self::[< $target_macro $target_bits >]:to_string:flatten:concatenate:unstringify as $target_macro;
                }
            };
        }
    };
}

/// An identity backed by a nonzero integer.
///
/// This trait does not guarantee uniqueness or that the referenced object remains
/// in storage.
pub trait Id: Hash + PartialEq + Eq + Copy + 'static {
    /// Numeric domain used for lossless construction and extraction of this identity.
    type BackingPrimitive: NonZeroable;

    /// Identity at the upper bound of the backing numeric domain.
    const MAX: Self;

    /// Identity at the least nonzero value of the backing numeric domain.
    const MIN: Self;

    /// Number of bits in the backing integer type.
    const BITS: u8 = Self::BackingPrimitive::BITS;

    /// Construct an identifier from its nonzero backing value.
    ///
    /// The result must expose the same value through [`as_nonzeroed`](Self::as_nonzeroed).
    fn raw(value: NonZeroed<Self::BackingPrimitive>) -> Self;

    /// Numeric identity for interchange with callers using primitive integers.
    /// Its value matches [`Self::as_nonzeroed`] and is never zero.
    #[inline]
    fn primitive(&self) -> Self::BackingPrimitive {
        self.as_nonzeroed().get()
    }

    /// Nonzero identity used to reconstruct this ID through [`Self::raw`].
    /// Implementations must preserve the value supplied at construction.
    fn as_nonzeroed(&self) -> NonZeroed<Self::BackingPrimitive>;
}

/// Distinguish IDs for different value types without changing their representation.
/// The tag carries no value of `T` and proves neither storage ownership nor membership.
#[repr(transparent)]
// NOTE(invariant): None.
pub struct TaggedId<T: ?Sized, I>
where
    I: Id,
{
    /// Identity used for comparison, hashing, and primitive conversion independently of `T`.
    target_id: I,

    /// Type distinction that requires no stored or owned value of `T`.
    _marker: PhantomData<fn() -> T>,
}

impl<T: ?Sized, I> TaggedId<T, I>
where
    I: Id,
{
    /// Erase the value-type distinction while preserving the original identity.
    #[inline]
    pub const fn unwrap(self) -> I {
        let Self { target_id, .. } = self;

        target_id
    }
}

impl<T: ?Sized, I> fmt::Display for TaggedId<T, I>
where
    I: Id,
    I: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let &Self { target_id, .. } = self;

        target_id.fmt(f)
    }
}

impl<T: ?Sized, I> TaggedId<T, I>
where
    I: Id,
{
    /// Associate an existing identity with `T` without validating storage membership.
    #[inline]
    pub const fn new(target_id: I) -> Self {
        Self {
            target_id,
            _marker: PhantomData,
        }
    }
}

impl<T: ?Sized, I: Ord> Ord for TaggedId<T, I>
where
    I: Id,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let &Self { target_id, .. } = self;

        let &Self {
            target_id: other_target_id,
            ..
        } = other;

        target_id.cmp(&other_target_id)
    }
}

impl<T: ?Sized, I: PartialOrd> PartialOrd for TaggedId<T, I>
where
    I: Id,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let &Self { target_id, .. } = self;

        let &Self {
            target_id: other_target_id,
            ..
        } = other;

        target_id.partial_cmp(&other_target_id)
    }
}

impl<T: ?Sized, I: hash::Hash> hash::Hash for TaggedId<T, I>
where
    I: Id,
{
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
        let Self { target_id, .. } = self;

        target_id.hash(state);
    }
}

impl<T: ?Sized, I: Eq> Eq for TaggedId<T, I> where I: Id {}

impl<T: ?Sized, I: PartialEq> PartialEq for TaggedId<T, I>
where
    I: Id,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let &Self { target_id, .. } = self;

        let &Self {
            target_id: other_target_id,
            ..
        } = other;

        target_id == other_target_id
    }
}

impl<T: ?Sized, I: fmt::Debug> fmt::Debug for TaggedId<T, I>
where
    I: Id,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let &Self { ref target_id, .. } = self;

        target_id.fmt(f)
    }
}

impl<T: ?Sized, I> Clone for TaggedId<T, I>
where
    I: Id,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized, I> Copy for TaggedId<T, I> where I: Id {}

impl<T: ?Sized + 'static, I> Id for TaggedId<T, I>
where
    I: Id,
{
    type BackingPrimitive = I::BackingPrimitive;

    const MAX: Self = Self::new(I::MAX);

    const MIN: Self = Self::new(I::MIN);

    #[inline]
    fn raw(value: NonZeroed<Self::BackingPrimitive>) -> Self {
        Self::new(I::raw(value))
    }

    #[inline]
    fn as_nonzeroed(&self) -> NonZeroed<Self::BackingPrimitive> {
        let &Self { target_id, .. } = self;

        target_id.as_nonzeroed()
    }
}

#[cfg(test)]
mod tests {
    use core::mem::size_of;

    use super::{Id as IdTrait, TaggedId};

    Id!(
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        become TestId
    );

    TestId!(8 become);

    #[test]
    fn tagged_id_preserves_inner_identity() {
        let id = TestId::MIN;
        let tagged = TaggedId::<str, TestId>::new(id);

        assert_eq!(tagged.unwrap(), id);
    }

    #[test]
    fn tagged_id_preserves_primitive_representation_without_storage_overhead() {
        let id = TestId::MAX;
        let tagged = TaggedId::<str, TestId>::new(id);

        assert_eq!(tagged.primitive(), id.primitive());
        assert_eq!(size_of::<TaggedId<str, TestId>>(), size_of::<TestId>());
    }
}
