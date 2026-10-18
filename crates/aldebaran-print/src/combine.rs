//! Deferred composition of printable values.
//!
//! [`Combine`] builds printable output without writing it. Aggregation shares
//! a printing context, sequencing keeps separate contexts, and repetition reuses
//! the same context for each rendering. The caller supplies context and sink
//! when the composed value is printed.

use core::fmt;

use crate::print::Print;

/// Deferred output composition for values that retain their own context requirements.
/// Builders preserve the subjects until rendering and add no separators.
pub trait Combine: Print {
    /// Concatenate two subjects that resolve against the same printing context.
    #[inline]
    fn aggregate<T>(self, other: T) -> impl Print<Context = Self::Context>
    where
        T: Print<Context = Self::Context>,
        Self: Sized,
    {
        Aggregate { left: self, right: other }
    }

    /// Concatenate subjects with independently supplied printing contexts.
    /// The composed context pairs the left subject's context with the right's.
    #[inline]
    fn sequence<T>(self, other: T) -> impl Print<Context = (Self::Context, T::Context)>
    where
        T: Print,
        Self: Sized,
    {
        Sequence { left: self, right: other }
    }

    /// Repeat rendering with shared context and no separators.
    /// A zero count writes nothing, and a formatting error stops further output.
    #[inline]
    fn times(self, count: u32) -> impl Print<Context = Self::Context>
    where
        Self: Sized,
    {
        Times { inner: self, count }
    }
}

impl<T> Combine for T where T: Print + ?Sized {}

/// Adjacent output from two subjects resolved against a shared context.
// NOTE(invariant): None.
struct Aggregate<T, U>
where
    T: Print,
    U: Print<Context = T::Context>,
{
    /// Value emitted first.
    left: T,

    /// Value emitted second.
    right: U,
}

impl<T, U> Print for Aggregate<T, U>
where
    T: Print,
    U: Print<Context = T::Context>,
{
    type Context = T::Context;

    #[inline]
    fn print_with_context<W>(&self, writer: &mut W, context: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        let Self { left, right } = self;

        left.print_with_context(writer, context)?;
        right.print_with_context(writer, context)
    }
}

/// Adjacent output whose paired contexts resolve the left and right subjects independently.
// NOTE(invariant): None.
struct Sequence<T, U>
where
    T: Print,
    U: Print,
{
    /// Value emitted first.
    left: T,

    /// Value emitted second.
    right: U,
}

impl<T, U> Print for Sequence<T, U>
where
    T: Print,
    U: Print,
{
    type Context = (T::Context, U::Context);

    #[inline]
    fn print_with_context<W>(&self, writer: &mut W, context: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        let Self { left, right } = self;

        let (left_context, right_context) = context;

        left.print_with_context(writer, left_context)?;
        right.print_with_context(writer, right_context)
    }
}

/// Repeated rendering of a retained subject with shared context and no separators.
// NOTE(invariant): None.
struct Times<T>
where
    T: Print,
{
    /// Value repeated by this adaptor.
    inner: T,

    /// Number of repetitions to emit.
    count: u32,
}

impl<T> Print for Times<T>
where
    T: Print,
{
    type Context = T::Context;

    #[inline]
    fn print_with_context<W>(&self, writer: &mut W, context: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        let Self { inner, count } = self;

        for _ in 0..*count {
            inner.print_with_context(writer, context)?;
        }

        Ok(())
    }
}
