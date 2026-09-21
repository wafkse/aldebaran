//! All unary operation traits and implementations.

use super::{Operable, unary_list};

/// Declares all unary operation traits.
macro_rules! unary {
    (
        $(
            $target_name:ident as $target_symbol:tt
        )+
    ) => {
        ::tokel::stream! {
            $(
                #[doc = concat!("The `", stringify!($target_name), "` operation.")]
                pub trait $target_name: Operable {
                    /// The output type.
                    type Output;

                    #[doc = concat!("Performs the `", stringify!($target_name), "` operation.")]
                    fn [< $target_name >]:case[[snake]](self) -> Self::Output;
                }
            )+
        }
    };
    () => {};
}

/// Implements all unary operation traits for a primitive.
macro_rules! impl_unary_for {
    (
        ($target_name:ident as $target_symbol:tt) as
        $(
            $target_type:ty
        )+
    ) => {
        ::tokel::stream! {
            $(
                impl $target_name for $target_type {
                    type Output = $target_type;

                    #[inline]
                    fn [< $target_name >]:case[[snake]] (self) -> Self::Output {
                        $target_symbol self
                    }
                }
            )+
        }
    };
}

/// Implements all unary operation traits for all primitive types.
macro_rules! impl_unary {
    (
        $(
            $target_name:ident as $target_op:tt
        )+
    ) => {
        $(
            $crate::primitive_list!(impl_unary_for => ($target_name as $target_op));
        )+
    };
}

macro_rules! impl_supertrait {
    (
        $(
            $target_name:ident as $_:tt
        )+
    ) => {
        ::tokel::stream! {
            /// A supertrait for all unary operations.
            pub trait UnOp:
                $(
                    $target_name +
                )+
                Operable
            {
                /// The output type for all underlying unary operations.
                type Output;
            }

            impl<T> UnOp for T
            where T:
                $(
                    $target_name +
                )+
                Operable
            {
                type Output = T;
            }
        }
    };
}

unary_list!(unary);
unary_list!(impl_unary);

unary_list!(impl_supertrait);

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
                #[test]
                fn [< unop_ [< $target_type >]:case[[snake]] _impl_ [< $target_integer >]:case[[snake]] >]:to_string:flatten:concatenate:unstringify() {
                    let target_left: $target_integer = random!($target_integer);

                    let target_expect: $target_integer = $target_operator target_left;

                    let target_value: $target_integer = target_left.[< $target_type >]:case[[snake]]();

                    assert_eq!(target_expect, target_value);
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
                    $target_name
                ),+
            };

            use const_random::const_random as random;


            $(
                $crate::primitive_list!($crate::op::unary::impl_tests => ($target_name as $target_operator));
            )+
        }
    };
}

#[allow(unused_imports)]
pub(self) use impl_tests;

unary_list!(impl_tests);
