//! Exclusive disjunction over two assertion values.
//!
//! [`Xor`] accepts exactly when one operand accepts. Semantic classification
//! succeeds only for a single selected branch and records that branch through
//! [`Select`](crate::select::Select).

use core::fmt;

use crate::{
    assert::Assert,
    choose::Choose,
    fmt::{BinaryConnective, Connective, Formatter, Precedence},
    input::Input,
    select::Select,
};

/// Exclusive logical disjunction between two assertions.
///
/// Exactly one operand must accept an input. Classification therefore records
/// the sole successful branch and rejects inputs accepted by both or neither.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Xor<A1, A2>(A1, A2);

impl<A1, A2> Xor<A1, A2> {
    /// The exclusive disjunction between two distinct assertions.
    #[inline]
    pub const fn one(lhs: A1, rhs: A2) -> Self {
        Self(lhs, rhs)
    }
}

impl<I, A1, A2> Assert<I> for Xor<A1, A2>
where
    I: Input,
    A1: Assert<I>,
    A2: Assert<I>,
{
    #[inline]
    fn assert(&self, input: I) -> bool {
        let Self(lhs, rhs) = self;

        lhs.assert(input) != rhs.assert(input)
    }

    #[inline]
    fn output_with<W, F>(target_value: &Self, writer: &mut W, target_precedence: Precedence) -> fmt::Result
    where
        W: fmt::Write,
        F: Formatter,
    {
        let Self(lhs, rhs) = target_value;

        F::start(writer, Connective::XOR, target_precedence)?;

        Assert::output_with::<W, F>(lhs, writer, Precedence::XOR)?;

        F::middle(writer, BinaryConnective::XOR, target_precedence)?;

        Assert::output_with::<W, F>(rhs, writer, Precedence::XOR)?;

        F::end(writer, Connective::XOR, target_precedence)?;

        Ok(())
    }
}

impl<I, A1, A2> Choose<I> for Xor<A1, A2>
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
        let rhs = rhs.choose(input);

        match (lhs, rhs) {
            (Some(lhs), None) => Some(Select::First(lhs)),
            (None, Some(rhs)) => Some(Select::Second(rhs)),
            _ => None,
        }
    }
}
