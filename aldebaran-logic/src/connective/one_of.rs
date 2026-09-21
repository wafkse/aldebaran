//! Fixed and borrowed disjunctions over homogeneous assertions.
//!
//! [`OneOf`] stores a compile-time array while [`Sliced`] borrows an arbitrary
//! slice. Both accept when any candidate accepts and semantic classification
//! returns the first candidate that can classify the input.

use core::fmt;
use core::ops::{Deref, DerefMut};

use crate::{
    assert::Assert,
    choose::Choose,
    fmt::{BinaryConnective, Connective, Formatter, Precedence},
    input::Input,
};

/// A N-ary logical disjunction between N distinct assertions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OneOf<A, const N: usize>([A; N]);

impl<A, const N: usize> OneOf<A, N> {
    /// Disjunt N distinct assertions.
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

        let _ = target_list
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

/// A `N`-ary logical disjunction between N distinct assertions of the same type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Sliced<'a, A>(&'a [A]);

impl<'a, A> Sliced<'a, A> {
    /// Disjunt `N` distinct assertions.
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

        let _ = target_list
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
