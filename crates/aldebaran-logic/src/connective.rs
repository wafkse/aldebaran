//! Structural composition of logical assertions.
//!
//! See [`Connective`] for further information.

mod and;
mod not;
mod one_of;
mod or;
mod pair;
mod r#ref;
mod xor;

pub use self::{
    and::And,
    not::Not,
    one_of::{OneOf, Sliced},
    or::Or,
    pair::Pair,
    r#ref::Ref,
    xor::Xor,
};

/// Structural constructors for logical assertion expressions.
pub trait Connective {
    /// Require two assertions to be satisfied simultaneously.
    ///
    /// When both operands classify the input, the result retains both
    /// classifications.
    #[inline]
    fn and<A>(self, other: A) -> And<Self, A>
    where
        Self: Sized,
    {
        And::them(self, other)
    }

    /// Require at least one of two assertions to be satisfied.
    ///
    /// When both operands classify the input, the left classification takes
    /// precedence.
    #[inline]
    fn or<A>(self, other: A) -> Or<Self, A>
    where
        Self: Sized,
    {
        Or::one(self, other)
    }

    /// Require exactly one of two assertions to be satisfied.
    ///
    /// Classification records the sole matching branch.
    #[inline]
    fn xor<A>(self, other: A) -> Xor<Self, A>
    where
        Self: Sized,
    {
        Xor::one(self, other)
    }

    /// Negate the current assertion expression without producing a classification.
    #[inline]
    fn not(self) -> Not<Self>
    where
        Self: Sized,
    {
        Not::that(self)
    }

    /// Borrow the current assertion expression without moving or cloning it.
    #[inline]
    fn borrowed(&self) -> Ref<'_, Self>
    where
        Self: Sized,
    {
        Ref::that(self)
    }
}

/// Every value can participate structurally in an assertion expression.
impl<A> Connective for A where A: ?Sized {}
