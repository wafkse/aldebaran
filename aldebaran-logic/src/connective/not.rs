//! Logical negation of one assertion value.
//!
//! [`Not`] transparently owns one assertion and reverses its acceptance result.
//! Formatting delegates to the wrapped assertion with negation precedence while
//! dereference access preserves the concrete underlying assertion type.

use core::{
    fmt,
    ops::{Deref, DerefMut},
};

use crate::{
    assert::Assert,
    fmt::{Connective, Formatter, Precedence},
    input::Input,
};

/// Logical negation of one assertion.
///
/// The wrapper retains the original assertion value while reversing only the
/// predicate result and adding the appropriate formatter precedence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(transparent)]
pub struct Not<A>(A);

impl<A> Not<A> {
    /// Negate the target assertion.
    #[inline]
    pub const fn that(one: A) -> Self {
        Self(one)
    }
}

impl<I, A> Assert<I> for Not<A>
where
    I: Input,
    A: Assert<I>,
{
    #[inline]
    fn assert(&self, input: I) -> bool {
        let Self(assertion) = self;

        !assertion.assert(input)
    }

    #[inline]
    fn output_with<W, F>(target_value: &Self, writer: &mut W, target_precedence: Precedence) -> fmt::Result
    where
        W: fmt::Write,
        F: Formatter,
    {
        let Self(target_value) = target_value;

        F::start(writer, Connective::NOT, target_precedence)?;

        Assert::output_with::<W, F>(target_value, writer, Precedence::NOT)?;

        F::end(writer, Connective::NOT, target_precedence)?;

        Ok(())
    }
}

impl<A> Deref for Not<A> {
    type Target = A;

    #[inline]
    fn deref(&self) -> &Self::Target {
        let Self(target_value) = self;

        target_value
    }
}

impl<A> DerefMut for Not<A> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        let &mut Self(ref mut target_value) = self;

        target_value
    }
}
