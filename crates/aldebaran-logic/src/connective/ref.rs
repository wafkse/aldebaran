//! Borrowed assertion values for compositions.

use core::{fmt, ops::Deref};

use crate::{
    assert::Assert,
    choose::Choose,
    fmt::{Formatter, Precedence},
    input::Input,
};

/// Includes an existing predicate in a composition without moving or cloning it.
/// Acceptance, classification, and formatting retain the referenced predicate's contracts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
// NOTE(invariant): None.
pub struct Ref<'a, A>(&'a A);

impl<'a, A> Ref<'a, A> {
    /// Keep `target_ref` available to its owner while composing its predicate.
    #[inline]
    pub const fn that(target_ref: &'a A) -> Self {
        Self(target_ref)
    }

    /// The composed predicate with its original lifetime, independent of this adapter's borrow.
    #[inline]
    pub const fn reference(&self) -> &'a A {
        let &Self(target_ref) = self;

        target_ref
    }
}

impl<'a, I, A> Assert<I> for Ref<'a, A>
where
    I: Input,
    A: Assert<I>,
{
    #[inline]
    fn assert(&self, input: I) -> bool {
        let &Self(assertion) = self;

        assertion.assert(input)
    }

    #[inline]
    fn output_with<W, F>(target_value: &Self, writer: &mut W, target_precedence: Precedence) -> fmt::Result
    where
        W: fmt::Write,
        F: Formatter,
    {
        let &Self(target_value) = target_value;

        Assert::output_with::<W, F>(target_value, writer, target_precedence)?;

        Ok(())
    }
}

impl<'a, A> Deref for Ref<'a, A> {
    type Target = A;

    #[inline]
    fn deref(&self) -> &Self::Target {
        let &Self(target_value) = self;

        target_value
    }
}

impl<I, A> Choose<I> for Ref<'_, A>
where
    I: Input,
    A: Choose<I>,
{
    type Unit = A::Unit;

    #[inline]
    fn choose(&self, input: I) -> Option<Self::Unit> {
        let &Self(assertion) = self;

        assertion.choose(input)
    }
}
