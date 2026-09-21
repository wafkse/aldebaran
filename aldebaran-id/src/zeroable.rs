//! A module providing a trait for working with generic `NonZero` types.
//!
//! While regular [`NonZero`] is generic, the trait it uses is private and
//! cannot be expressed as a bound, therefore, this module provides a public
//! interface for these types.
//!
//! In reality, this still uses the compiler-provided `NonZero` type, but
//! wraps it around in a new-type.
//!
//! See the [`NonZeroed`] type and the [`NonZeroable`] trait for more
//! information.

use core::{
    fmt, hash,
    num::NonZero,
    ops::{Deref, DerefMut},
};

use aldebaran_primitive::macros::primitive_list;

mod private {
    /// A sealed trait to prevent external implementations for
    /// non-[`NonZero<T>`] types.
    pub trait Sealed {}
}

/// An abstract trait for types that can be zeroed.
///
/// Analogous to the internal [`ZeroablePrimitive`](core::num::ZeroablePrimitive) trait
/// found in the [`NonZero`] implementation.
pub trait NonZeroable: private::Sealed + Sized {
    /// The concrete, non-generic [`NonZero`] type.
    ///
    /// This is used to wrap around it in the [`NonZeroed`] new-type.
    ///
    /// The trait [`OperateNonZero`] is implemented for this type.
    type Type: OperateNonZero<Self>;
}

macro_rules! nonzeroable {
    () => {};
    (
        $(
            $target_type:ty
        )*
    ) => {
        $(
            impl private::Sealed for $target_type {}
            impl private::Sealed for NonZero<$target_type> {}

            impl NonZeroable for $target_type {
                type Type = NonZero<$target_type>;
            }

            impl From<NonZeroed<$target_type>> for $target_type {
                #[inline]
                fn from(value: NonZeroed<$target_type>) -> Self {
                    value.get()
                }
            }

            impl OperateNonZero<$target_type> for NonZero<$target_type> {
                const MIN: Self = NonZero::<$target_type>::MIN;

                const MAX: Self = NonZero::<$target_type>::MAX;

                #[inline]
                fn new(value: $target_type) -> Option<Self> {
                    NonZero::new(value)
                }

                #[inline]
                unsafe fn new_unchecked(value: $target_type) -> Self {
                    unsafe {
                        NonZero::new_unchecked(value)
                    }
                }

                #[inline]
                fn get(&self) -> $target_type {
                    NonZero::get(*self)
                }
            }
        )*
    };
}

primitive_list!(nonzeroable);

/// A newtype wrapper for `NonZero` types that can never be zeroed.
///
/// Even if this is a newtype, it is still `repr(transparent)` and is still
/// niche-optimized.
///
/// This both [`Deref`]s and [`DerefMut`]s to the inner [`NonZero`] type,
/// therefore, it can be used as a drop-in replacement.
#[repr(transparent)]
pub struct NonZeroed<T>(<T as NonZeroable>::Type)
where
    T: NonZeroable;

impl<T> fmt::Debug for NonZeroed<T>
where
    T: NonZeroable,
    T::Type: fmt::Debug,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self(inner) = self;

        inner.fmt(f)
    }
}

impl<T> fmt::Display for NonZeroed<T>
where
    T: NonZeroable,
    T::Type: fmt::Display,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self(inner) = self;

        inner.fmt(f)
    }
}

impl<T> Clone for NonZeroed<T>
where
    T: NonZeroable,
    T::Type: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let &Self(ref inner) = self;

        Self(inner.clone())
    }
}

impl<T> Copy for NonZeroed<T>
where
    T: NonZeroable,
    T::Type: Copy,
{
}

impl<T> PartialEq for NonZeroed<T>
where
    T: NonZeroable,
    T::Type: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let &Self(ref lhs) = self;
        let &Self(ref rhs) = other;

        lhs == rhs
    }
}

impl<T> Eq for NonZeroed<T>
where
    T: NonZeroable,
    T::Type: Eq,
{
}

impl<T> PartialOrd for NonZeroed<T>
where
    T: NonZeroable,
    T::Type: PartialOrd,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        let &Self(ref lhs) = self;
        let &Self(ref rhs) = other;

        lhs.partial_cmp(rhs)
    }
}

impl<T> Ord for NonZeroed<T>
where
    T: NonZeroable,
    T::Type: Ord,
{
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        let &Self(ref lhs) = self;
        let &Self(ref rhs) = other;

        lhs.cmp(rhs)
    }
}

impl<T> hash::Hash for NonZeroed<T>
where
    T: NonZeroable,
    T::Type: hash::Hash,
{
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        let &Self(ref inner) = self;

        inner.hash(state)
    }
}

impl<T> Default for NonZeroed<T>
where
    T: NonZeroable,
    T::Type: Default,
{
    #[inline]
    fn default() -> Self {
        Self(T::Type::MIN)
    }
}

impl<T> Deref for NonZeroed<T>
where
    T: NonZeroable,
{
    type Target = <T as NonZeroable>::Type;

    #[inline]
    fn deref(&self) -> &Self::Target {
        let &Self(ref inner) = self;

        inner
    }
}

impl<T> DerefMut for NonZeroed<T>
where
    T: NonZeroable,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        let &mut Self(ref mut inner) = self;

        inner
    }
}

impl<T> NonZeroed<T>
where
    T: NonZeroable,
{
    /// The maximum value that this [`NonZeroed`] can have.
    pub const MAX: Self = Self::raw(T::Type::MAX);

    /// The minimum value that this [`NonZeroed`] can have.
    pub const MIN: Self = Self::raw(T::Type::MIN);
}

impl<T> NonZeroed<T>
where
    T: NonZeroable,
{
    /// A new [`NonZeroed`] value that is created from a value that may be zero.
    #[inline]
    pub fn new(value: T) -> Option<Self> {
        T::Type::new(value).map(Self)
    }

    /// A new [`NonZeroed`] value that is created from a value that is known to
    /// be non-zero.
    ///
    /// # Safety
    ///
    /// The value must be non-zero. Otherwise, you risk undefined behavior due
    /// to niche optimizations.
    #[inline]
    pub unsafe fn new_unchecked(value: T) -> Self {
        // SAFETY: upheld by the caller
        let inner = unsafe { T::Type::new_unchecked(value) };

        Self(inner)
    }

    /// A new [`NonZeroed`] value that is created from its raw inner value, i.e,
    /// `core`'s `NonZero` type.
    #[inline]
    pub const fn raw(target_value: <T as NonZeroable>::Type) -> Self {
        Self(target_value)
    }
}

impl<T> NonZeroed<T>
where
    T: NonZeroable,
{
    /// Retrieve the inner value of the [`NonZeroed`] type.
    #[inline]
    pub fn get(&self) -> T {
        let &Self(ref inner) = self;

        inner.get()
    }

    /// Retrieve the inner [`NonZero`] type contained in this wrapper.
    #[inline]
    pub fn into_inner(self) -> T::Type {
        let Self(inner) = self;

        inner
    }
}

/// An trait that exposes the individual [`NonZero`] operations as part of a
/// larger trait.
pub trait OperateNonZero<T>: private::Sealed + Sized {
    /// The minimum value that can be represented by the [`NonZero`] type, i.e,
    /// `1`.
    const MIN: Self;

    /// The maximum value that can be represented by the [`NonZero`] type.
    const MAX: Self;

    /// A [`NonZero`] value that checks for zero.
    fn new(value: T) -> Option<Self>;

    /// A new [`NonZero`] value that does not check for zero.
    ///
    /// # Safety
    ///
    /// The value must be non-zero. Otherwise, you may risk random
    /// transmutations due to niche optimizations.
    unsafe fn new_unchecked(value: T) -> Self;

    /// Fetch the inner value of the [`NonZero`] type.
    fn get(&self) -> T;
}
