//! Identifier traits, generated identifier newtypes, and typed identifier tags.
//!
//! [`trait@Id`] defines the common nonzero primitive representation used by identifier
//! types. The `Id!` macro generates fixed-width newtypes, while [`TaggedId`]
//! attaches a compile-time target type without changing the stored identifier.

use core::{
    cmp, fmt,
    hash::{self, Hash},
    marker::PhantomData,
};

use aldebaran_primitive::prelude::Primitive;

use crate::zeroable::{NonZeroable, NonZeroed};

/// A helper macro that expands to a subordinated macro that aids in the
/// generation of newtype identifiers.
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
        #[doc = concat!("Expands to a new `", stringify!($target_macro), "Id[N]` new-type using `N` bits as storage.")]
        macro_rules! $target_macro {
            ($target_bits:literal) => {
                $crate::reexport::stream! {
                    #[doc = concat!("A ", stringify!($target_bits), "-bit identifier new-type.")]
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

/// A trait for types that identify arbitrary structures.
pub trait Id: Hash + PartialEq + Eq + Copy + 'static {
    /// The primitive type that this identifier is based on.
    ///
    /// This is usually a small-sized integer, such as [`u16`], or [`u32`].
    type BackingPrimitive: Primitive + NonZeroable;

    /// The maximum value that is possible for this [`trait@Id`].
    const MAX: Self;

    /// The minimum value that is possible for this [`trait@Id`].
    const MIN: Self;

    /// The number of bits in the underlying storage type.
    const BITS: u8 = Self::BackingPrimitive::BITS;

    /// A raw-constructed [`trait@Id`] from its [`NonZeroed`] primitive.
    fn raw(value: NonZeroed<Self::BackingPrimitive>) -> Self;

    /// The backing primitive value of this [`trait@Id`].
    #[inline]
    fn primitive(&self) -> Self::BackingPrimitive {
        self.as_nonzeroed().get()
    }

    /// Reinterpret this [`trait@Id`] as a [`NonZeroed`] value.
    fn as_nonzeroed(&self) -> NonZeroed<Self::BackingPrimitive>;
}

/// A tagged [`trait@Id`] that is associated with a specific type.
///
/// This type serves as a hint to the reader regarding which type the [`trait@Id`]
/// is associated with.
#[repr(transparent)]
pub struct TaggedId<T: ?Sized, I>
where
    I: Id,
{
    /// The target [`trait@Id`] that is tagged with a specific type.
    target_id: I,

    /// A marker to indicate the type of the target [`trait@Id`].
    // TODO: use new Phantom.. style types to indicate variance explicitly
    _marker: PhantomData<fn() -> T>,
}

impl<T: ?Sized, I> TaggedId<T, I>
where
    I: Id,
{
    /// Extract the inner [`trait@Id`] from this [`TaggedId`].
    #[inline]
    pub const fn into_inner(self) -> I {
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
    /// Tag the target [`trait@Id`] with a specific type.
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
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
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
        let &Self { target_id, _marker } = self;

        Self { target_id, _marker }
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

        assert_eq!(tagged.into_inner(), id);
    }

    #[test]
    fn tagged_id_preserves_primitive_representation_without_storage_overhead() {
        let id = TestId::MAX;
        let tagged = TaggedId::<str, TestId>::new(id);

        assert_eq!(tagged.primitive(), id.primitive());
        assert_eq!(size_of::<TaggedId<str, TestId>>(), size_of::<TestId>());
    }
}
