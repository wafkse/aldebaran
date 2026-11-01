//! Fixed and borrowed disjunctions over homogeneous assertions.
//!
//! See [`OneOf`] and [`Sliced`] for further information.

use core::fmt;
use core::ops::{Deref, DerefMut};

use crate::{
    assert::Assert,
    choose::Choose,
    fmt::{BinaryConnective, Connective, Formatter, Precedence},
    input::Input,
};

/// A logical disjunction between `N` assertions.
///
/// This assertion accepts an input if any of its assertions accepts it. When used
/// with [`Choose`], it returns the value produced by the first matching assertion
/// in array order, so earlier assertions take priority when more than one matches.
///
/// If the array is empty, no input is accepted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// NOTE(invariant): None.
pub struct OneOf<A, const N: usize>([A; N]);

impl<A, const N: usize> OneOf<A, N> {
    /// Create a disjunction from the given assertions.
    #[inline]
    pub const fn these(target_value: [A; N]) -> Self {
        Self(target_value)
    }

    /// Determine the underlying storage of the target assertions.
    #[inline]
    pub const fn storage(&self) -> &[A; N] {
        let Self(storage) = self;

        storage
    }
}

impl<I, A, const N: usize> Assert<I> for OneOf<A, N>
where
    I: Input,
    A: Assert<I>,
{
    #[inline]
    fn assert(&self, input: I) -> bool {
        let Self(assertions) = self;

        assertions.iter().any(|assertion| assertion.assert(input))
    }

    #[inline]
    fn output_with<W, F>(target_value: &Self, writer: &mut W, target_precedence: Precedence) -> fmt::Result
    where
        W: fmt::Write,
        F: Formatter,
    {
        let Self(target_list) = target_value;

        F::start(writer, Connective::OR, target_precedence)?;

        for target_value in target_list.iter().take(target_list.len().saturating_sub(1)) {
            Assert::output_with::<W, F>(target_value, writer, Precedence::OR)?;

            F::middle(writer, BinaryConnective::OR, target_precedence)?;
        }

        let _: Option<()> = target_list
            .last()
            .map(|target_value| Assert::output_with::<W, F>(target_value, writer, Precedence::OR))
            .transpose()?;

        F::end(writer, Connective::OR, target_precedence)?;

        Ok(())
    }
}

impl<A, const N: usize> Deref for OneOf<A, N> {
    type Target = [A; N];

    #[inline]
    fn deref(&self) -> &Self::Target {
        let Self(target_list) = self;

        target_list
    }
}

impl<A, const N: usize> DerefMut for OneOf<A, N> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        let Self(target_list) = self;

        target_list
    }
}

/// A logical disjunction between assertions borrowed from a slice.
///
/// The assertions are checked in slice order, so [`Choose`] returns the value
/// produced by the first matching assertion when more than one matches. If the
/// slice is empty, no input is accepted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
// NOTE(invariant): None.
pub struct Sliced<'a, A>(&'a [A]);

impl<'a, A> Sliced<'a, A> {
    /// Create a disjunction from the given assertions.
    #[inline]
    pub const fn these(target_value: &'a [A]) -> Self {
        Self(target_value)
    }

    /// Determine the underlying storage of the target assertions.
    #[inline]
    pub const fn storage(&self) -> &[A] {
        let Self(storage) = self;

        storage
    }
}

impl<'a, I, A> Assert<I> for Sliced<'a, A>
where
    I: Input,
    A: Assert<I>,
{
    #[inline]
    fn assert(&self, input: I) -> bool {
        let Self(assertions) = self;

        assertions.iter().any(|assertion| assertion.assert(input))
    }

    #[inline]
    fn output_with<W, F>(target_value: &Self, writer: &mut W, target_precedence: Precedence) -> fmt::Result
    where
        W: fmt::Write,
        F: Formatter,
    {
        let Self(target_list) = target_value;

        F::start(writer, Connective::OR, target_precedence)?;

        for target_value in target_list.iter().take(target_list.len().saturating_sub(1)) {
            Assert::output_with::<W, F>(target_value, writer, Precedence::OR)?;

            F::middle(writer, BinaryConnective::OR, target_precedence)?;
        }

        let _: Option<()> = target_list
            .last()
            .map(|target_value| Assert::output_with::<W, F>(target_value, writer, Precedence::OR))
            .transpose()?;

        F::end(writer, Connective::OR, target_precedence)?;

        Ok(())
    }
}

impl<'a, A> Deref for Sliced<'a, A> {
    type Target = [A];

    #[inline]
    fn deref(&self) -> &Self::Target {
        let Self(target_list) = self;

        target_list
    }
}

impl<I, A, const N: usize> Choose<I> for OneOf<A, N>
where
    I: Input,
    A: Choose<I>,
{
    type Unit = A::Unit;

    #[inline]
    fn choose(&self, input: I) -> Option<Self::Unit> {
        let Self(assertions) = self;

        assertions.iter().find_map(|assertion| assertion.choose(input))
    }
}

impl<I, A> Choose<I> for Sliced<'_, A>
where
    I: Input,
    A: Choose<I>,
{
    type Unit = A::Unit;

    #[inline]
    fn choose(&self, input: I) -> Option<Self::Unit> {
        let Self(assertions) = self;

        assertions.iter().find_map(|assertion| assertion.choose(input))
    }
}
