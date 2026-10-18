//! Assertion descriptions usable with Rust's display and debug formatting.
//! The adapters select an input type and notation policy, with optional precedence
//! for embedding a description in a surrounding expression.

use core::{fmt, marker, ops::Deref};

use crate::{
    assert::Assert,
    fmt::{Formatter, Precedence},
    input::Input,
    prelude::DefaultFormatter,
};

/// Makes an assertion's description available to [`fmt::Display`] without requiring
/// that trait on the assertion itself. `F` selects notation with no surrounding operator.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[repr(transparent)]
// NOTE(invariant): None.
pub struct Displayed<'a, A, I, F = DefaultFormatter>(&'a A, marker::PhantomData<(I, F)>)
where
    A: Assert<I>,
    I: Input,
    F: Formatter;

impl<'a, A, I, F> Displayed<'a, A, I, F>
where
    A: Assert<I>,
    I: Input,
    F: Formatter,
{
    /// Describe this predicate independently of any surrounding connective.
    #[inline]
    pub const fn now(target_value: &'a A) -> Self {
        Self(target_value, marker::PhantomData)
    }
}

impl<'a, A, I, F> Deref for Displayed<'a, A, I, F>
where
    A: Assert<I>,
    I: Input,
    F: Formatter,
{
    type Target = A;

    #[inline]
    fn deref(&self) -> &Self::Target {
        let &Self(target_value, ..) = self;

        target_value
    }
}

impl<'a, A, I, F> fmt::Display for Displayed<'a, A, I, F>
where
    A: Assert<I>,
    I: Input,
    F: Formatter,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let &Self(target_value, ..) = self;

        Assert::output_with::<fmt::Formatter, F>(target_value, f, Precedence::None)
    }
}

/// Makes an assertion's description available to [`fmt::Display`] inside an expression.
/// `F` selects notation and uses the supplied precedence to preserve grouping.
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
// NOTE(invariant): None.
pub struct DisplayedWith<'a, A, I, F = DefaultFormatter>
where
    I: Input,
    A: Assert<I>,
    F: Formatter,
{
    /// Assertion whose description is rendered.
    target_value: &'a A,

    /// Binding strength of the expression surrounding this description.
    precedence: Precedence,

    /// The input type and formatter selected for this adapter.
    marker: marker::PhantomData<(F, I)>,
}

impl<'a, A, I, F> DisplayedWith<'a, A, I, F>
where
    I: Input,
    A: Assert<I>,
    F: Formatter,
{
    /// Embed this predicate's display description in the supplied operator context.
    #[inline]
    pub const fn now(target_value: &'a A, precedence: Precedence) -> Self {
        Self {
            target_value,
            precedence,
            marker: marker::PhantomData,
        }
    }
}

impl<'a, A, I, F> Deref for DisplayedWith<'a, A, I, F>
where
    I: Input,
    A: Assert<I>,
    F: Formatter,
{
    type Target = A;

    #[inline]
    fn deref(&self) -> &Self::Target {
        let &Self { target_value, .. } = self;

        target_value
    }
}

impl<'a, A, I, F> fmt::Display for DisplayedWith<'a, A, I, F>
where
    A: Assert<I>,
    I: Input,
    F: Formatter,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let &Self {
            target_value, precedence, ..
        } = self;

        Assert::output_with::<fmt::Formatter, F>(target_value, f, precedence)
    }
}
/// Makes an assertion's description available to [`fmt::Debug`] without requiring
/// that trait on the assertion itself. `F` selects notation with no surrounding operator.
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[repr(transparent)]
// NOTE(invariant): None.
pub struct Debugged<'a, A, I, F = DefaultFormatter>(&'a A, marker::PhantomData<(I, F)>)
where
    A: Assert<I>,
    I: Input,
    F: Formatter;

impl<'a, A, I, F> Debugged<'a, A, I, F>
where
    A: Assert<I>,
    I: Input,
    F: Formatter,
{
    /// Use the predicate's description as debug output without a surrounding connective.
    #[inline]
    pub const fn now(target_value: &'a A) -> Self {
        Self(target_value, marker::PhantomData)
    }
}

impl<'a, A, I, F> Deref for Debugged<'a, A, I, F>
where
    A: Assert<I>,
    I: Input,
    F: Formatter,
{
    type Target = A;

    #[inline]
    fn deref(&self) -> &Self::Target {
        let &Self(target_value, ..) = self;

        target_value
    }
}

impl<'a, A, I, F> fmt::Debug for Debugged<'a, A, I, F>
where
    A: Assert<I>,
    I: Input,
    F: Formatter,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let &Self(target_value, ..) = self;

        Assert::output_with::<fmt::Formatter, F>(target_value, f, Precedence::None)
    }
}

/// Makes an assertion's description available to [`fmt::Debug`] inside an expression.
/// `F` selects notation and uses the supplied precedence to preserve grouping.
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
// NOTE(invariant): None.
pub struct DebuggedWith<'a, A, I, F = DefaultFormatter>
where
    I: Input,
    A: Assert<I>,
    F: Formatter,
{
    /// Assertion whose description is rendered.
    target_value: &'a A,

    /// Binding strength of the expression surrounding this description.
    precedence: Precedence,

    /// The input type and formatter selected for this adapter.
    marker: marker::PhantomData<(F, I)>,
}

impl<'a, A, I, F> DebuggedWith<'a, A, I, F>
where
    I: Input,
    A: Assert<I>,
    F: Formatter,
{
    /// Embed this predicate's debug description in the supplied operator context.
    #[inline]
    pub const fn now(target_value: &'a A, precedence: Precedence) -> Self {
        Self {
            target_value,
            precedence,
            marker: marker::PhantomData,
        }
    }
}

impl<'a, A, I, F> Deref for DebuggedWith<'a, A, I, F>
where
    I: Input,
    A: Assert<I>,
    F: Formatter,
{
    type Target = A;

    #[inline]
    fn deref(&self) -> &Self::Target {
        let &Self { target_value, .. } = self;

        target_value
    }
}

impl<'a, A, I, F> fmt::Debug for DebuggedWith<'a, A, I, F>
where
    A: Assert<I>,
    I: Input,
    F: Formatter,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let &Self {
            target_value, precedence, ..
        } = self;

        Assert::output_with::<fmt::Formatter, F>(target_value, f, precedence)
    }
}
