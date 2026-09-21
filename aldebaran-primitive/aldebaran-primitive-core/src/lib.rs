//! Abstraction over all primitive types.
//!
//! This crate provides a set of traits and macros for working with primitive
//! types in a generic fashion.

#![cfg_attr(not(test), no_std)]
#![forbid(
    unsafe_code,
    missing_docs,
    missing_debug_implementations,
    clippy::missing_panics_doc,
    clippy::missing_safety_doc
)]

use aldebaran_primitive_macro::{is_signed, primitive_list, signed, signed_list, unsigned, unsigned_list};

pub mod cast;
pub mod op;

use cast::Cast;
use op::Operate;

mod private {
    /// A sealed trait.
    ///
    /// This trait cannot be implemented outside this crate.
    pub trait Sealed {}

    impl<T> Sealed for &T where T: Sealed {}
}

/// Implements the primitive trait for the target primitive types.
macro_rules! impl_primitive {
    (
        $target_trait:ty as
        $(
            $target_type:ident
        )+
    ) => {
        $(
            impl $crate::private::Sealed for $target_type {}

            impl $target_trait for $target_type {
                type Signed = signed!($target_type);
                type Unsigned = unsigned!($target_type);

                const SIGNED: bool = is_signed!($target_type);

                const MIN: Self = <$target_type>::MIN;
                const MAX: Self = <$target_type>::MAX;
                const BITS: u8 = <$target_type>::BITS as _;

                const ZERO: Self = 0;
                const ONE: Self = 1;
            }
        )+
    };
}

/// Implement a sealed marker trait for a set of types.
macro_rules! marker {
    (
        $target_trait:ident as $($target_type:ident)*
    ) => {
        $(
            impl $target_trait for $target_type {}
        )*
    };
}

primitive_list!(impl_primitive => Primitive);

/// Marker trait for signed types.
pub trait Signed: crate::private::Sealed {}

/// Marker trait for unsigned types.
pub trait Unsigned: crate::private::Sealed {}

signed_list!(marker => Signed);
unsigned_list!(marker => Unsigned);

/// A trait resembling an integer primitive type.
///
/// This is a common interface for all primitive types that are integers, and
/// provides a set of common operations that can be performed on them.
pub trait Primitive: Operate + crate::private::Sealed
where
    Self: Copy,
{
    /// The signed version of the primitive type.
    type Signed: Primitive + Cast<Self>;

    /// The unsigned version of the primitive type.
    type Unsigned: Primitive + Cast<Self>;

    /// Whether this numeric primitive is signed or not.
    const SIGNED: bool;

    /// The minimum value of the primitive type.
    const MIN: Self;

    /// The maximum value of the primitive type.
    const MAX: Self;

    /// The zero value of the primitive type.
    const ZERO: Self;

    /// The one value of the primitive type.
    const ONE: Self;

    /// The number of bits used to represent the primitive type.
    const BITS: u8;
}
