//! Typed composition of assertions.
//!
//! Combinators retain structured formatting and, where supported, the
//! classification types of their operands.

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

/// Composition syntax that preserves each operand's assertion and classification
/// contracts. The resulting value is usable for an input only when its operands
/// support that input.
pub trait Connective {
    /// Require both operands to accept the same input, retaining both classifications.
    #[inline]
    fn and<A>(self, other: A) -> And<Self, A>
    where
        Self: Sized,
    {
        And::them(self, other)
    }

    /// Accept either operand, preferring the left classification when both match.
    #[inline]
    fn or<A>(self, other: A) -> Or<Self, A>
    where
        Self: Sized,
    {
        Or::one(self, other)
    }

    /// Accept exactly one operand, retaining which operand classified the input.
    #[inline]
    fn xor<A>(self, other: A) -> Xor<Self, A>
    where
        Self: Sized,
    {
        Xor::one(self, other)
    }

    /// Accept the complement of this predicate without a classification result.
    #[inline]
    fn not(self) -> Not<Self>
    where
        Self: Sized,
    {
        Not::that(self)
    }

    /// Include this predicate in a composition without moving or cloning it.
    #[inline]
    fn borrowed(&self) -> Ref<'_, Self>
    where
        Self: Sized,
    {
        Ref::that(self)
    }
}

/// All types implement this trait. Its methods require a sized receiver.
impl<A> Connective for A where A: ?Sized {}
