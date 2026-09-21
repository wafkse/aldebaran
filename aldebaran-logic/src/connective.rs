//! Structural composition of logical assertions.
//!
//! Connective wrappers preserve the concrete assertion types while composing
//! conjunction, disjunction, exclusive disjunction, negation, borrowing, and
//! fixed-arity alternatives. [`Connective`] provides the ergonomic constructors.

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
    #[inline]
    fn and<A>(self, other: A) -> And<Self, A>
    where
        Self: Sized,
    {
        And::them(self, other)
    }

    /// Require at least one of two assertions to be satisfied.
    #[inline]
    fn or<A>(self, other: A) -> Or<Self, A>
    where
        Self: Sized,
    {
        Or::one(self, other)
    }

    /// Require exactly one of two assertions to be satisfied.
    #[inline]
    fn xor<A>(self, other: A) -> Xor<Self, A>
    where
        Self: Sized,
    {
        Xor::one(self, other)
    }

    /// Negate the current assertion expression.
    #[inline]
    fn not(self) -> Not<Self>
    where
        Self: Sized,
    {
        Not::that(self)
    }

    /// Borrow the current assertion expression.
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
