//! Top-level select functionality.
//!
//! This module provides a family of types that are used to select between two
//! or more distinct [`Assert`] types.

use core::fmt;

use crate::{assert::Assert, choose::Choose, fmt::Precedence, input::Input, prelude::Formatter};

/// A select operation between two distinct assertions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Select<A1, A2> {
    /// The first type variant.
    First(A1),

    /// The second type variant.
    Second(A2),
}

impl<I, A1, A2> Assert<I> for Select<A1, A2>
where
    I: Input,
    A1: Assert<I>,
    A2: Assert<I>,
{
    #[inline]
    fn assert(&self, input: I) -> bool {
        match self {
            Self::First(assertion) => assertion.assert(input),
            Self::Second(assertion) => assertion.assert(input),
        }
    }

    #[inline]
    fn output_with<W, F>(target_value: &Self, writer: &mut W, target_precedence: Precedence) -> fmt::Result
    where
        W: fmt::Write,
        F: Formatter,
    {
        match target_value {
            Self::First(target_assert) => Assert::output_with::<W, F>(target_assert, writer, target_precedence),
            Self::Second(target_assert) => Assert::output_with::<W, F>(target_assert, writer, target_precedence),
        }
    }
}

impl<I, A1, A2> Choose<I> for Select<A1, A2>
where
    I: Input,
    A1: Choose<I>,
    A2: Choose<I>,
{
    type Unit = Select<A1::Unit, A2::Unit>;

    #[inline]
    fn choose(&self, input: I) -> Option<Self::Unit> {
        match self {
            Self::First(assertion) => assertion.choose(input).map(Select::First),
            Self::Second(assertion) => assertion.choose(input).map(Select::Second),
        }
    }
}

/// A macro that implements predicate and classification behavior for a target select type.
///
/// Select types are assertions that can be any of a fixed number of downstream assertions.
#[macro_export]
macro_rules! Selector {
    () => {};
    (
        @impl for $target_name:ident
        $(
            <
                $(
                    $target_param_ty:ident,
                ),*

                $(,)?
            >
        )?


    ) => {

    };
    (
        $(
           #[$target_attr:meta]
        )*
        $target_vis:vis enum $target_name:ident
        $(
            <
                $(
                    $target_param_ty:ident,
                ),*

                $(,)?
            >
        )?
        [
            $(
                $(
                    #[$target_variant_attr:meta]
                )*
                $target_variant:ident($target_variant_ty:ty)
            ),+

            $(,)?
        ]
    ) => {
        $(
            #[$target_attr]
        )*
        $target_vis enum $target_name
        $(
            <
                $(
                    $target_param_ty,
                ),*

                $(,)?
            >
        )?
        {
            $(
                $(
                    #[$target_variant_attr]
                )*
                $target_variant($target_variant_ty),
            )*
        }

        impl
        <
            I,

            $(
                $(
                    $target_param_ty,
                )*
            )?
        > $crate::assert::Assert<I> for $target_name
        $(
            <
                $(
                    $target_param_ty,
                ),*

                $(,)?
            >
        )?
        where
            I: $crate::input::Input,
            $($target_variant_ty: $crate::assert::Assert<I>,)?
        {
            #[inline]
            fn assert(&self, input: I) -> bool {
                match self {
                    $(
                        Self::$target_variant(assertion) => $crate::assert::Assert::assert(assertion, input),
                    )*
                }
            }

            #[inline]
            fn output_with<W, F>(target_value: &Self, writer: &mut W, target_precedence: $crate::fmt::Precedence) -> ::core::fmt::Result
            where
                W: ::core::fmt::Write,
                F: $crate::fmt::Formatter,
            {
                match target_value {
                    $(
                        Self::$target_variant(target_assert) => $crate::assert::Assert::output_with::<W, F>(target_assert, writer, target_precedence),
                    )*
                }
            }
        }

        impl
        <
            I,

            $(
                $(
                    $target_param_ty,
                )*
            )?
        > $crate::choose::Choose<I> for $target_name
        $(
            <
                $(
                    $target_param_ty,
                ),*

                $(,)?
            >
        )?
        where
            I: $crate::input::Input,
            Self: ::core::clone::Clone,
            $($target_variant_ty: $crate::assert::Assert<I>,)?
        {
            type Unit = Self;

            #[inline]
            fn choose(&self, input: I) -> ::core::option::Option<Self::Unit> {
                if $crate::assert::Assert::assert(self, input) {
                    ::core::option::Option::Some(::core::clone::Clone::clone(self))
                } else {
                    ::core::option::Option::None
                }
            }
        }
    };
}

#[doc(inline)]
pub use crate::Selector;
