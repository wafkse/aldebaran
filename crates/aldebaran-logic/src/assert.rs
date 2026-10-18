//! Predicates with descriptions suitable for diagnostics. [`Assert`] combines
//! acceptance with formatting, while [`IntoAssert`] and [`AsAssert`] let APIs
//! accept owned or borrowed forms of the predicate they require.

use core::fmt;

use crate::{
    fmt::{Format, Formatter, Precedence},
    input::Input,
};

/// An acceptance rule with a description suitable for an expectation message.
/// The description belongs to the rule and does not require an input value.
pub trait Assert<I>
where
    I: Input,
{
    /// Establish whether the input belongs to the set described by this rule.
    fn assert(&self, input: I) -> bool;

    /// Format this predicate at top level, with no surrounding connective.
    #[inline]
    fn output<W, F>(target_value: &Self, writer: &mut W) -> fmt::Result
    where
        W: fmt::Write,
        F: Formatter,
    {
        Assert::output_with::<W, F>(target_value, writer, Precedence::None)
    }

    /// Format this predicate in the surrounding connective's precedence context.
    ///
    /// A formatter can use that context to group a lower-precedence predicate.
    fn output_with<W, F>(target_value: &Self, writer: &mut W, target_precedence: Precedence) -> fmt::Result
    where
        W: fmt::Write,
        F: Formatter;
}

/// Treats a formattable input value as an assertion for exact equality.
///
/// The formatted description comes from [`Format`].
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

/// Converts a supplied value to the assertion type required by an API.
///
/// Values already of type `A` pass through unchanged. Consumers can implement
/// conversions for other forms they accept.
pub trait IntoAssert<I, A>
where
    I: Input,
    A: Assert<I>,
{
    /// Transfer ownership into the predicate form required by the receiving API.
    fn into_assert(self) -> A;
}

/// Leaves an assertion unchanged when it already has the selected type.
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

/// Borrows a value as an assertion of the selected type.
///
/// The blanket implementations accept an assertion value or a reference to
/// that assertion, which lets APIs inspect both forms without consuming it.
pub trait AsAssert<I, A>
where
    I: Input,
    A: Assert<I>,
{
    /// Expose the predicate for inspection without transferring ownership.
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
