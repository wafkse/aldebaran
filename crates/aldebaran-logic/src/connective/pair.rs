//! Assertions applied independently to tuple components.

use core::fmt;

use crate::{
    assert::Assert,
    choose::Choose,
    fmt::{BinaryConnective, Connective, Formatter, Precedence},
    input::Input,
};

/// Requires each tuple component to satisfy its corresponding predicate.
/// Successful classification retains both results in their input positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
// NOTE(invariant): None.
pub struct Pair<A1, A2>(A1, A2);

impl<A1, A2> Pair<A1, A2> {
    /// Assign `first` and `second` to the corresponding tuple positions.
    #[inline]
    pub const fn new(first: A1, second: A2) -> Self {
        Self(first, second)
    }
}

impl<I1, I2, A1, A2> Assert<(I1, I2)> for Pair<A1, A2>
where
    I1: Input,
    I2: Input,
    A1: Assert<I1>,
    A2: Assert<I2>,
{
    #[inline]
    fn assert(&self, input: (I1, I2)) -> bool {
        let Self(first, second) = self;

        let (first_input, second_input) = input;

        first.assert(first_input) && second.assert(second_input)
    }

    #[inline]
    fn output_with<W, F>(target_value: &Self, writer: &mut W, target_precedence: Precedence) -> fmt::Result
    where
        W: fmt::Write,
        F: Formatter,
    {
        let Self(first, second) = target_value;

        F::start(writer, Connective::AND, target_precedence)?;

        Assert::output_with::<W, F>(first, writer, Precedence::AND)?;

        F::middle(writer, BinaryConnective::AND, target_precedence)?;

        Assert::output_with::<W, F>(second, writer, Precedence::AND)?;

        F::end(writer, Connective::AND, target_precedence)
    }
}

impl<I1, I2, A1, A2> Choose<(I1, I2)> for Pair<A1, A2>
where
    I1: Input,
    I2: Input,
    A1: Choose<I1>,
    A2: Choose<I2>,
{
    type Unit = Pair<A1::Unit, A2::Unit>;

    #[inline]
    fn choose(&self, input: (I1, I2)) -> Option<Self::Unit> {
        let Self(first, second) = self;

        let (first_input, second_input) = input;

        let first = first.choose(first_input)?;
        let second = second.choose(second_input)?;

        Some(Pair::new(first, second))
    }
}
