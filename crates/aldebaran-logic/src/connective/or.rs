//! Disjunction over a shared input with left-biased classification.

use core::fmt;

use crate::{
    assert::Assert,
    choose::Choose,
    fmt::{BinaryConnective, Connective, Formatter, Precedence},
    input::Input,
    select::Select,
};

/// Accepts either operand on the same input and preserves the selected branch
/// in [`Select`]. When both operands classify the input, the left result wins.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
// NOTE(invariant): None.
pub struct Or<A1, A2>(A1, A2);

impl<A1, A2> Or<A1, A2> {
    /// Prefer `lhs` when both predicates can classify the same input.
    #[inline]
    pub const fn one(lhs: A1, rhs: A2) -> Self {
        Self(lhs, rhs)
    }
}

impl<I, A1, A2> Assert<I> for Or<A1, A2>
where
    I: Input,
    A1: Assert<I>,
    A2: Assert<I>,
{
    #[inline]
    fn assert(&self, input: I) -> bool {
        let Self(lhs, rhs) = self;

        lhs.assert(input) || rhs.assert(input)
    }

    #[inline]
    fn output_with<W, F>(target_value: &Self, writer: &mut W, target_precedence: Precedence) -> fmt::Result
    where
        W: fmt::Write,
        F: Formatter,
    {
        let Self(lhs, rhs) = target_value;

        F::start(writer, Connective::OR, target_precedence)?;

        Assert::output_with::<W, F>(lhs, writer, Precedence::OR)?;

        F::middle(writer, BinaryConnective::OR, target_precedence)?;

        Assert::output_with::<W, F>(rhs, writer, Precedence::OR)?;

        F::end(writer, Connective::OR, target_precedence)?;

        Ok(())
    }
}

impl<I, A1, A2> Choose<I> for Or<A1, A2>
where
    I: Input,
    A1: Choose<I>,
    A2: Choose<I>,
{
    type Unit = Select<A1::Unit, A2::Unit>;

    #[inline]
    fn choose(&self, input: I) -> Option<Self::Unit> {
        let Self(lhs, rhs) = self;

        let lhs = lhs.choose(input);

        match lhs {
            Some(unit) => Some(Select::First(unit)),
            None => rhs.choose(input).map(Select::Second),
        }
    }
}
