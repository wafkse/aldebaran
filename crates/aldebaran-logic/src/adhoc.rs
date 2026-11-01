//! Formatting adaptors for using assertions in ordinary Rust diagnostics.
//!
//! See [`Displayed`], [`DisplayedWith`], [`Debugged`], and [`DebuggedWith`] for
//! further information.
//!
//! These adaptors borrow the assertion and do not allocate or copy it.

use core::{fmt, marker, ops::Deref};

use crate::{
    assert::Assert,
    fmt::{Formatter, Precedence},
    input::Input,
    prelude::DefaultFormatter,
};

/// A zero-cost ad-hoc adaptor that makes a [`Assert`] implementation be able to be used as a regular [`fmt::Display`] implementor.
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
    /// Make the target [`Assert`] `A` be able to be used as a regular [`fmt::Display`] implementor.
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

/// A zero-cost ad-hoc adaptor that makes a [`Assert`] implementation be able to be used as a regular [`fmt::Display`] implementor.
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
// NOTE(invariant): None.
pub struct DisplayedWith<'a, A, I, F = DefaultFormatter>
where
    I: Input,
    A: Assert<I>,
    F: Formatter,
{
    /// The target [`Assert`] implementation.
    target_value: &'a A,

    /// The surrounding connective precedence used to format the target assertion.
    precedence: Precedence,

    /// Marker field to ensure that both [`Input`] `I` and [`Formatter`] `F` are deemed part of this type.
    marker: marker::PhantomData<(F, I)>,
}

impl<'a, A, I, F> DisplayedWith<'a, A, I, F>
where
    I: Input,
    A: Assert<I>,
    F: Formatter,
{
    /// Make the target [`Assert`] `A` be able to be used as a regular [`fmt::Display`] implementor.
    ///
    /// Takes an additional [`Precedence`] to use for the respective [`Assert::output_with`] method.
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
/// A zero-cost ad-hoc adaptor that makes a [`Assert`] implementation be able to be used as a regular [`fmt::Debug`] implementor.
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
    /// Make the target [`Assert`] `A` be able to be used as a regular [`fmt::Debug`] implementor.
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

/// A zero-cost ad-hoc adaptor that makes a [`Assert`] implementation be able to be used as a regular [`fmt::Debug`] implementor.
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
// NOTE(invariant): None.
pub struct DebuggedWith<'a, A, I, F = DefaultFormatter>
where
    I: Input,
    A: Assert<I>,
    F: Formatter,
{
    /// The target [`Assert`] implementation.
    target_value: &'a A,

    /// The surrounding connective precedence used to format the target assertion.
    precedence: Precedence,

    /// Marker field to ensure that both [`Input`] `I` and [`Formatter`] `F` are deemed part of this type.
    marker: marker::PhantomData<(F, I)>,
}

impl<'a, A, I, F> DebuggedWith<'a, A, I, F>
where
    I: Input,
    A: Assert<I>,
    F: Formatter,
{
    /// Make the target [`Assert`] `A` be able to be used as a regular [`fmt::Debug`] implementor.
    ///
    /// Takes an additional [`Precedence`] to use for the respective [`Assert::output_with`] method.
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
