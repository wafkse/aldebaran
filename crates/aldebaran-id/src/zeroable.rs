//! A module providing a trait for working with generic [`NonZero`] types.
//!
//! See [`NonZeroable`], [`NonZeroed`], and [`OperateNonZero`] for further information.

use core::{
    cmp::Ordering,
    fmt, hash,
    num::NonZero,
    ops::{Deref, DerefMut},
};

/// Seals the primitive and nonzero operations to supported integer types.
mod private {
    /// A sealed trait to prevent external implementations for
    /// types other than the supported integer and [`NonZero`] types.
    pub trait Sealed {}
}

/// An abstract trait for types that can be represented by a nonzero integer.
///
/// The implementations are sealed to the built-in signed and unsigned integer
/// primitives.
pub trait NonZeroable: private::Sealed + Sized {
    /// The concrete, non-generic [`NonZero`] type associated with this primitive.
    ///
    /// This is used as the representation inside [`NonZeroed`].
    type Type: OperateNonZero<Self>;

    /// Number of bits in the built-in integer representation.
    const BITS: u8;
}

/// Implements the nonzero abstraction for the supplied primitive integer types.
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

                const BITS: u8 = <$target_type>::BITS as u8;
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
                    // SAFETY: The caller guarantees that `value` is nonzero.
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

nonzeroable!(i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize);

/// A newtype wrapper for [`NonZero`] values that can never be zero.
///
/// This type is `repr(transparent)` and retains the niche optimization of its
/// inner value. It implements [`Deref`] and [`DerefMut`] to the inner [`NonZero`] type,
/// so safe mutable access preserves the nonzero guarantee.
#[repr(transparent)]
// NOTE(invariant): None.
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
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
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
    fn cmp(&self, other: &Self) -> Ordering {
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
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
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
    /// Create a new [`NonZeroed`] value from a primitive that may be zero.
    ///
    /// Returns [`None`] if the value is zero.
    #[inline]
    pub fn new(value: T) -> Option<Self> {
        T::Type::new(value).map(Self)
    }

    /// Create a new [`NonZeroed`] value from a primitive that is known to be nonzero.
    ///
    /// # Safety
    ///
    /// `value` must be nonzero. Passing zero violates the validity requirement
    /// of the underlying [`NonZero`] value and is undefined behavior.
    #[inline]
    pub unsafe fn new_unchecked(value: T) -> Self {
        // SAFETY: The caller guarantees the nonzero input required by `new_unchecked`.
        let inner = unsafe { T::Type::new_unchecked(value) };

        Self(inner)
    }

    /// Create a [`NonZeroed`] value from the corresponding [`NonZero`] value.
    #[inline]
    pub const fn raw(target_value: <T as NonZeroable>::Type) -> Self {
        Self(target_value)
    }
}

impl<T> NonZeroed<T>
where
    T: NonZeroable,
{
    /// Retrieve the inner primitive value of the [`NonZeroed`] type.
    #[inline]
    pub fn get(&self) -> T {
        let &Self(ref inner) = self;

        inner.get()
    }

    /// Retrieve the inner [`NonZero`] type contained in this wrapper.
    #[inline]
    pub fn unwrap(self) -> T::Type {
        let Self(inner) = self;

        inner
    }
}

/// An interface exposing the individual operations of a [`NonZero`] integer type.
pub trait OperateNonZero<T>: private::Sealed + Sized {
    /// The minimum value that can be represented by the [`NonZero`] type.
    const MIN: Self;

    /// The maximum value that can be represented by the [`NonZero`] type.
    const MAX: Self;

    /// Create a [`NonZero`] value, checking that the input is nonzero.
    fn new(value: T) -> Option<Self>;

    /// Construct a [`NonZero`] value without checking its input.
    ///
    /// # Safety
    ///
    /// `value` must be nonzero. Passing zero violates the validity requirement
    /// of [`NonZero`] and is undefined behavior.
    unsafe fn new_unchecked(value: T) -> Self;

    /// Fetch the inner primitive value of the [`NonZero`] type.
    fn get(&self) -> T;
}
