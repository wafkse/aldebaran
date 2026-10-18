//! Nonzero integer values with a primitive type parameter.
//!
//! [`NonZeroable`] associates supported primitives with [`NonZero`] representations,
//! allowing [`NonZeroed`] to preserve nonzero validity during generic construction
//! and mutable access.

use core::{
    cmp::Ordering,
    fmt, hash,
    num::NonZero,
    ops::{Deref, DerefMut},
};

/// Limits generic nonzero support to built-in integer representations.
mod private {
    /// Eligibility for the primitive-to-nonzero correspondence used by this module.
    pub trait Sealed {}
}

/// Maps an integer primitive to its corresponding [`NonZero`] type.
///
/// Implementations are sealed to the signed and unsigned integer primitives.
pub trait NonZeroable: private::Sealed + Sized {
    /// Representation that excludes zero while preserving the primitive's numeric domain.
    type Type: OperateNonZero<Self>;

    /// Number of bits in the built-in integer representation.
    const BITS: u8;
}

/// Built-in integer families eligible for generic nonzero construction and extraction.
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

/// A nonzero value parameterized by its primitive integer type.
/// Dereferencing exposes the corresponding [`NonZero`] representation, so safe
/// mutable access cannot introduce zero.
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
    /// The greatest nonzero value of the backing integer type.
    pub const MAX: Self = Self::raw(T::Type::MAX);

    /// The least nonzero value of the backing integer type.
    pub const MIN: Self = Self::raw(T::Type::MIN);
}

impl<T> NonZeroed<T>
where
    T: NonZeroable,
{
    /// Create a nonzero value, returning [`None`] for zero.
    #[inline]
    pub fn new(value: T) -> Option<Self> {
        T::Type::new(value).map(Self)
    }

    /// Construct a nonzero value without checking its input.
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

    /// Preserve an existing nonzero value without requiring another validity check.
    #[inline]
    pub const fn raw(target_value: <T as NonZeroable>::Type) -> Self {
        Self(target_value)
    }
}

impl<T> NonZeroed<T>
where
    T: NonZeroable,
{
    /// Numeric value for primitive integer operations, still guaranteed to be nonzero.
    #[inline]
    pub fn get(&self) -> T {
        let &Self(ref inner) = self;

        inner.get()
    }

    /// Preserve nonzero validity when transferring the value to standard integer APIs.
    #[inline]
    pub fn into_inner(self) -> T::Type {
        let Self(inner) = self;

        inner
    }
}

/// Construction and extraction shared by built-in [`NonZero`] integer types.
/// Implementations are sealed so generic consumers retain standard nonzero validity.
pub trait OperateNonZero<T>: private::Sealed + Sized {
    /// The least nonzero value representable by the primitive integer type.
    const MIN: Self;

    /// The maximum value that can be represented by the [`NonZero`] type.
    const MAX: Self;

    /// Return [`None`] for zero and a [`NonZero`] value otherwise.
    fn new(value: T) -> Option<Self>;

    /// Construct a [`NonZero`] value without checking its input.
    ///
    /// # Safety
    ///
    /// `value` must be nonzero. Passing zero violates the validity requirement
    /// of [`NonZero`] and is undefined behavior.
    unsafe fn new_unchecked(value: T) -> Self;

    /// Numeric value represented by this nonzero integer, unchanged by extraction.
    fn get(&self) -> T;
}
