//! All binary operation traits and implementations.

use super::{Operable, binary_list};

/// Declares all binary operation traits.
macro_rules! binary {
    (
        $(
            $target_name:ident as $target_symbol:tt
        )+
    ) => {
        ::tokel::stream! {
            $(
                #[doc = concat!("The `", stringify!($target_name), "` operation.")]
                pub trait $target_name: Operable {
                    /// The type of the right operand.
                    type Operand;
                    /// The output type.
                    type Output;

                    #[doc = concat!("Performs the `", stringify!($target_name), "` operation.")]
                    fn [< $target_name >]:case[[snake]](self, target_right: Self::Operand) -> Self::Output;
                }
            )+
        }
    };
    () => {};
}

/// Implements all binary operation traits for a primitive.
macro_rules! impl_binary_for {
    (
        ($target_name:ident as $target_symbol:tt) as
        $(
            $target_type:ty
        )+
    ) => {
        ::tokel::stream! {
            $(
                impl $target_name for $target_type {
                    type Operand = Self;
                    type Output = Self;

                    #[inline]
                    fn [< $target_name >]:case[[snake]](self, target_right: Self::Operand) -> Self::Output {
                        self $target_symbol target_right
                    }
                }
            )+
        }
    };
}

/// Implements all binary operation traits for all primitive types.
macro_rules! impl_binary {
    (
        $(
            $target_name:ident as $target_op:tt
        )+
    ) => {
        $(
            $crate::primitive_list!(impl_binary_for => ($target_name as $target_op));
        )+
    };
}

macro_rules! impl_supertrait {
    (
        $(
            $target_name:ident as $_l:tt
        )+
    ) => {
        ::tokel::stream! {
            /// Supertrait for all binary operations.
            pub trait BinOp:
                $(
                    $target_name +
                )+
                Operable
            {
                /// The type of the left operand.
                type Operand;
                /// The type of the output.
                type Output;
            }

            impl<T> BinOp for T
            where T:
                $(
                    $target_name +
                )+
                Operable
            {
                type Operand = T;
                type Output = T;
            }
        }
    };
}

binary_list!(binary);
binary_list!(impl_binary);

binary_list!(impl_supertrait);

macro_rules! impl_tests {
    (
        ($target_type:ty as $target_operator:tt) as
        $(
            $target_integer:ty
        )+

        $(,)?
    ) => {
        ::tokel::stream! {
            $(
                // NOTE: We check both for panic-behavior and correctness.
                #[test]
                fn [< binop_ [< $target_type >]:case[[snake]] _impl_ [< $target_integer >]:case[[snake]] >]:to_string:flatten:concatenate:unstringify() {
                    use std::{mem::discriminant, panic::catch_unwind};

                    let target_left: $target_integer = random!($target_integer);
                    let target_right: $target_integer = random!($target_integer);

                    let target_expect: Result<$target_integer, _> = catch_unwind(|| {
                        target_left $target_operator target_right
                    });

                    let target_value: Result<$target_integer, _> = catch_unwind(|| {
                        target_left.[< $target_type >]:case[[snake]](target_right)
                    });

                    match (target_expect, target_value) {
                        (Ok(expect), Ok(value)) => assert_eq!(expect, value),
                        (target_expect, target_value) => assert_eq!(discriminant(&target_expect), discriminant(&target_value))
                    }
                }
            )+
        }
    };
    (
        $(
            $target_name:ident as $target_operator:tt
        )+
    ) => {
        #[cfg(test)]
        mod tests {
            use super::{
                $(
                    $target_name,
                )+
            };

            use const_random::const_random as random;


            $(
                $crate::primitive_list!($crate::op::binary::impl_tests => ($target_name as $target_operator));
            )+
        }
    };
}

#[allow(unused_imports)]
pub(self) use impl_tests;

binary_list!(impl_tests);
