//! Facilities for casting between primitive types in a generic fashion.
//!
//! See [`Cast`] for more information.
//!
//! This module also provides the [`LosslessCast`] trait,
//! which is a subtrait of [`Cast`] that guarantees that the cast is lossless.

/// Implement primitive `as` casts for each listed target type.
macro_rules! impl_castable {
    (
        $target_type:ty as
        $(
            $target_out:ty
        )+

        $(,)?
    ) => {
        $(
            #[expect(clippy::allow_attributes)]
            #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
            impl Cast<$target_out> for $target_type {
                #[inline]
                fn cast(self) -> $target_out {
                    self as $target_out
                }
            }
        )+
    };
    (
        $(
            $target_type:ty
        )+

        $(,)?
    ) => {
        $(
            $crate::primitive_list!($crate::cast::impl_castable => $target_type);
        )+
    };
}

/// Implement the [`LosslessCast`] trait for the target primitive types.
///
/// This assumes that the input types are postfix variants and are ordered by
/// size.
///
/// A lossless cast is a cast that does not lose any information.
///
/// To achieve this, the tra
macro_rules! impl_lossless_castable {
    (
        use
    ) => { /* base case */ };
    (
        use

        $target_type:tt

        $(
            $target_out:tt
        )*
    ) => {
        ::tokel::stream! {
            $(
                impl LosslessCast<[< u $target_out >]:to_string:flatten:concatenate:unstringify>
                    for [< u $target_type >]:to_string:flatten:concatenate:unstringify
                {
                    #[inline]
                    fn lossless_cast(
                        self,
                    ) -> [< u $target_out >]:to_string:flatten:concatenate:unstringify {
                        <[< u $target_out >]:to_string:flatten:concatenate:unstringify>::from(self)
                    }
                }

                impl LosslessCast<[< i $target_out >]:to_string:flatten:concatenate:unstringify>
                    for [< i $target_type >]:to_string:flatten:concatenate:unstringify
                {
                    #[inline]
                    fn lossless_cast(
                        self,
                    ) -> [< i $target_out >]:to_string:flatten:concatenate:unstringify {
                        <[< i $target_out >]:to_string:flatten:concatenate:unstringify>::from(self)
                    }
                }
            )*
        }

        impl_lossless_castable! {
            use
            $(
                $target_out
            )*
        }
    };
    (
        $(
            $target_type:tt
        )+
    ) => {
        impl_lossless_castable! {
            use $(
                $target_type
            )+
        }
    };
}

/// Generate boundary tests for each primitive cast.
macro_rules! impl_tests {
    (
        $target_type:ty as
        $(
            $target_out:ty
        )+

        $(,)?
    ) => {
        ::tokel::stream! {
            $(
                #[test]
                #[expect(clippy::allow_attributes)]
                #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
                fn [< cast_ [< $target_type >]:case[[snake]] _as_ [< $target_out >]:case[[snake]] _max >]:to_string:flatten:concatenate:unstringify() {
                    let target = <$target_type>::MAX;
                    let result: $target_out = target.cast();

                    assert_eq!(result, target as $target_out);
                }

                #[test]
                #[expect(clippy::allow_attributes)]
                #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
                fn [< cast_ [< $target_type >]:case[[snake]] _as_ [< $target_out >]:case[[snake]] _min >]:to_string:flatten:concatenate:unstringify() {
                    let target = <$target_type>::MIN;
                    let result: $target_out = target.cast();

                    assert_eq!(result, target as $target_out);
                }
            )+
        }
    };
    (
        $(
            $target_type:ty
        )+

        $(,)?
    ) => {
        #[cfg(test)]
        mod tests {
            use super::{Cast, LosslessCast};

            #[test]
            fn lossless_unsigned_boundaries() {
                for value in [u8::MIN, u8::MAX] {
                    let widened: u128 = value.lossless_cast();

                    assert_eq!(widened, u128::from(value));
                }
            }

            #[test]
            fn lossless_signed_boundaries() {
                for value in [i8::MIN, -1, 0, i8::MAX] {
                    let widened: i128 = value.lossless_cast();

                    assert_eq!(widened, i128::from(value));
                }
            }

            $(
                $crate::primitive_list!($crate::cast::impl_tests => $target_type);
            )+
        }
    };
}

use impl_castable;

#[cfg(test)]
pub(crate) use impl_tests;

use aldebaran_primitive_macro::sized_list;

use crate::{Primitive, primitive_list, private};

/// A trait for casting between primitive types.
// FIXME: Make this a sealed trait.
pub trait Cast<O>: private::Sealed {
    /// Explicitly cast the value to the target type `O`.
    fn cast(self) -> O;
}

/// A trait that provides a lossless cast.
pub trait LosslessCast<O>: Cast<O> {
    /// Losslessly cast the value to the target type `O`.
    fn lossless_cast(self) -> O;
}

impl<I, O> Cast<O> for &I
where
    I: Cast<O> + Copy,
{
    #[inline]
    fn cast(self) -> O {
        <I as Cast<O>>::cast(*self)
    }
}

impl<I, O> LosslessCast<O> for &I
where
    I: LosslessCast<O> + Copy,
{
    #[inline]
    fn lossless_cast(self) -> O {
        <I as LosslessCast<O>>::lossless_cast(*self)
    }
}

/// A trait for casts that allow for the possibility of failure.
///
/// This is very much alike to [`LosslessCast`], but allows for the possibility
/// of some conversions to succeed and others to fail.
pub trait TryCast<O>: Cast<O> {
    /// Explicitly attempt to cast the value to the target type `O`.
    fn try_cast(self) -> Option<O>;
}

impl<T, O> TryCast<O> for T
where
    T: Primitive + PartialEq,
    O: Primitive + PartialEq,
    T: Cast<O> + Copy,
    O: Cast<T> + Copy,
{
    #[inline]
    fn try_cast(self) -> Option<O> {
        let result = self.cast();

        let back: Self = result.cast();

        (self == back).then_some(result)
    }
}

primitive_list!(impl_castable);

sized_list!(impl_lossless_castable);

primitive_list!(impl_tests);
