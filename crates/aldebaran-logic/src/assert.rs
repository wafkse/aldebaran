//! A module containing the [`Assert`] trait and its implementations.
//!
//! Note that the [`Assert`] trait has brother traits very much alike to
//! [`Into`] or [`AsRef`], but specialized for [`Assert`]s.

use core::fmt;

use crate::{
    fmt::{Format, Formatter, Precedence},
    input::Input,
};

/// A trait for types that test whether an arbitrary input `I` satisfies a property.
pub trait Assert<I>
where
    I: Input,
{
    /// Determine whether the input satisfies this assertion.
    fn assert(&self, input: I) -> bool;

    /// Pretty-print this [`Assert`] using the target [`Formatter`].
    #[inline]
    fn output<W, F>(target_value: &Self, writer: &mut W) -> fmt::Result
    where
        W: fmt::Write,
        F: Formatter,
    {
        Assert::output_with::<W, F>(target_value, writer, Precedence::None)
    }

    /// Pretty-print this [`Assert`] using a [`Formatter`] and a top-level [`Precedence`].
    fn output_with<W, F>(target_value: &Self, writer: &mut W, target_precedence: Precedence) -> fmt::Result
    where
        W: fmt::Write,
        F: Formatter;
}

/// Blanket implementation of [`Assert`] for types that are both [`Input`] and
/// [`Format`].
impl<I> Assert<I> for I
where
    I: Input,
    I: Format,
{
    #[inline]
    fn assert(&self, input: I) -> bool {
        self == &input
    }

    #[inline]
    fn output_with<W, F>(target_value: &Self, writer: &mut W, _: Precedence) -> fmt::Result
    where
        W: fmt::Write,
        F: Formatter,
    {
        I::output(target_value, writer)
    }
}

/// A trait that is analogous to [`Into`] but for exclusive use with [`Assert`].
///
/// The most prevalent use case for this trait is to allow simultaneous use of
/// values that can be coerced to some assertion type, and values that are
/// already of that assertion type.
///
/// Note that there is no such thing as `FromAssert` because conversion from an
/// arbitrary type of some kind to an assertion type is not trivial.
pub trait IntoAssert<I, A>
where
    I: Input,
    A: Assert<I>,
{
    /// Convert the type into an [`Assert`].
    fn into_assert(self) -> A;
}

/// Blanket implementation of [`IntoAssert`] for types that are already
/// [`Assert`]s.
impl<I, A> IntoAssert<I, A> for A
where
    I: Input,
    A: Assert<I>,
{
    #[inline]
    fn into_assert(self) -> A {
        self
    }
}

/// A trait that is analogous to [`AsRef`] but for exclusive use with
/// [`Assert`].
///
/// The most prevalent use case for this trait is to allow simultaneous use of
/// both by-value and by-reference [`Assert`] types.
pub trait AsAssert<I, A>
where
    I: Input,
    A: Assert<I>,
{
    /// Convert the input reference into a reference to an [`Assert`] type `A`.
    fn as_assert(&self) -> &A;
}

impl<I, T> AsAssert<I, T> for T
where
    I: Input,
    T: Assert<I>,
{
    #[inline]
    fn as_assert(&self) -> &T {
        self
    }
}

impl<I, T> AsAssert<I, T> for &T
where
    I: Input,
    T: Assert<I>,
{
    #[inline]
    fn as_assert(&self) -> &T {
        self
    }
}
