//! Borrowed assertion adaptation without changing assertion semantics.
//!
//! [`Ref`] lets a borrowed assertion participate anywhere an owned connective
//! value is expected. Predicate evaluation, classification, and formatter output
//! are forwarded directly to the referenced assertion.

use core::{fmt, ops::Deref};

use crate::{
    assert::Assert,
    choose::Choose,
    fmt::{Formatter, Precedence},
    input::Input,
};

/// An adapter type for borrowed [`Assert`]s.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct Ref<'a, A>(&'a A);

impl<'a, A> Ref<'a, A> {
    /// Use the target reference as the assertion.
    #[inline]
    pub const fn that(target_ref: &'a A) -> Self {
        Self(target_ref)
    }

    /// Retrieve the target reference that is borrowed for `'a`.
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
