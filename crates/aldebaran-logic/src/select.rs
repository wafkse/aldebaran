//! Branch identity for heterogeneous assertion results. [`Select`] preserves
//! which side matched, while [`Selector!`](macro@crate::Selector) gives caller-named
//! alternatives a shared assertion interface.

use core::fmt;

use crate::{assert::Assert, choose::Choose, fmt::Precedence, input::Input, prelude::Formatter};

/// Preserves branch identity when alternatives produce different classification types.
/// As an assertion itself, only the selected variant's predicate applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Select<A1, A2> {
    /// Left operand's classification, as selected by disjunction.
    First(A1),

    /// Right operand's classification, as selected by disjunction.
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

/// Declares an enum whose variants hold assertions for the same input type.
///
/// Only the active variant's predicate applies. Payloads must implement [`Assert`],
/// and the enum must implement [`Clone`] because successful [`Choose`] results
/// preserve the enum value rather than the payload's classification.
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
