//! Conjunction over a shared input, retaining both successful classifications.

use core::fmt;

use crate::{
    assert::Assert,
    choose::Choose,
    fmt::{BinaryConnective, Connective, Formatter, Precedence},
    input::Input,
};

/// Requires both operands to accept the same input. Successful classification
/// retains the left and right results as an [`And`] in the same order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
// NOTE(invariant): None.
pub struct And<A1, A2>(A1, A2);

impl<A1, A2> And<A1, A2> {
    /// Require both `lhs` and `rhs` to accept the same input.
    #[inline]
    pub const fn them(lhs: A1, rhs: A2) -> Self {
        Self(lhs, rhs)
    }
}

impl<I, A1, A2> Assert<I> for And<A1, A2>
where
    I: Input,
    A1: Assert<I>,
    A2: Assert<I>,
{
    #[inline]
    fn assert(&self, input: I) -> bool {
        let Self(lhs, rhs) = self;

        lhs.assert(input) && rhs.assert(input)
    }

    #[inline]
    fn output_with<W, F>(target_value: &Self, writer: &mut W, target_precedence: Precedence) -> fmt::Result
    where
        W: fmt::Write,
        F: Formatter,
    {
        let Self(lhs, rhs) = target_value;

        F::start(writer, Connective::AND, target_precedence)?;

        Assert::output_with::<W, F>(lhs, writer, Precedence::AND)?;

        F::middle(writer, BinaryConnective::AND, target_precedence)?;

        Assert::output_with::<W, F>(rhs, writer, Precedence::AND)?;

        F::end(writer, Connective::AND, target_precedence)?;

        Ok(())
    }
}

impl<I, A1, A2> Choose<I> for And<A1, A2>
where
    I: Input,
    A1: Choose<I>,
    A2: Choose<I>,
{
    type Unit = And<A1::Unit, A2::Unit>;

    #[inline]
    fn choose(&self, input: I) -> Option<Self::Unit> {
        let Self(lhs, rhs) = self;

        let lhs = lhs.choose(input);
        let rhs = rhs.choose(input);

        match (lhs, rhs) {
            (Some(lhs), Some(rhs)) => Some(And::them(lhs, rhs)),
            _ => None,
        }
    }
}
