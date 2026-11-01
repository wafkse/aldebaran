//! Deferred composition of printable values.
//!
//! See [`Combine`] for further information.

use core::fmt;

use crate::print::Print;

/// Extension operations for composing printable values.
///
/// Builders retain the subjects until rendering and add no separators.
pub trait Combine: Print {
    /// Print `self` followed by `other` using one shared context.
    #[inline]
    fn aggregate<T>(self, other: T) -> impl Print<Context = Self::Context>
    where
        T: Print<Context = Self::Context>,
        Self: Sized,
    {
        Aggregate { left: self, right: other }
    }

    /// Print `self` followed by `other` using independent contexts.
    ///
    /// The composed context pairs the left subject's context with the right's.
    #[inline]
    fn sequence<T>(self, other: T) -> impl Print<Context = (Self::Context, T::Context)>
    where
        T: Print,
        Self: Sized,
    {
        Sequence { left: self, right: other }
    }

    /// Print `self` exactly `count` times using the same context each time.
    ///
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

/// Two printable values that share one context.
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

/// Two printable values with independent contexts.
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

/// One printable value repeated a fixed number of times.
///
/// Each repetition uses the same context, and no separator is added.
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
