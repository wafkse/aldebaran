//! Context-aware printing into formatting sinks.
//!
//! [`Print`] writes a value using external context such as an interner or arena,
//! so the value need not retain that data or implement [`fmt::Display`] itself.
//! The display adapters make the same printing behavior available to standard
//! formatting when a context can be supplied or created by default.
//!
//! Output goes incrementally to a [`fmt::Write`] sink. Because earlier writes
//! are not rolled back, a later formatting error can leave partially written
//! output in that sink.

use core::{
    convert::Infallible,
    fmt,
    ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo},
};

/// Text rendering with externally supplied data such as an interner or arena.
///
/// The context belongs to the caller, allowing stored values to remain separate
/// from the data needed to display them. Output errors can leave a written
/// prefix in the sink, with no rollback required by this contract.
#[diagnostic::on_unimplemented(message = "{Self} is not printable")]
pub trait Print {
    /// Data consulted while producing this value's textual representation.
    type Context;

    /// Render using a newly created default context.
    /// Prefer an explicit context when output depends on caller-owned storage.
    #[inline]
    fn print<W>(&self, writer: &mut W) -> fmt::Result
    where
        W: fmt::Write,
        Self::Context: Default,
    {
        let context = Self::Context::default();

        self.print_with_context(writer, &context)
    }

    /// Render with caller-owned context into the supplied sink.
    /// Implementations should propagate sink errors without implying output rollback.
    fn print_with_context<W>(&self, writer: &mut W, context: &Self::Context) -> fmt::Result
    where
        W: fmt::Write;

    /// Use this rendering in standard formatting with a fresh default context each time.
    #[inline]
    #[must_use]
    fn display(&self) -> impl fmt::Display + '_
    where
        Self::Context: Default,
    {
        DisplayDefault(self)
    }

    /// Use this rendering in standard formatting while borrowing the same context.
    /// The adapter retains both borrows until formatting has finished.
    #[inline]
    #[must_use]
    fn display_with_context<'a>(&'a self, context: &'a Self::Context) -> impl fmt::Display + 'a {
        DisplayWith {
            target_value: self,
            context,
        }
    }
}

/// Standard formatting that creates a fresh default context for each rendering.
#[repr(transparent)]
// NOTE(invariant): None.
struct DisplayDefault<'a, P>(&'a P)
where
    P: Print + ?Sized;

impl<P> fmt::Display for DisplayDefault<'_, P>
where
    P: Print + ?Sized,
    P::Context: Default,
{
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self(target_value) = self;

        target_value.print(formatter)
    }
}

/// Standard formatting tied to caller-owned context for the lifetime of the adapter.
// NOTE(invariant): None.
struct DisplayWith<'a, P>
where
    P: Print + ?Sized,
{
    /// Borrowed subject whose textual form is resolved at formatting time.
    target_value: &'a P,

    /// Caller-owned data retained alongside the subject for subsequent formatting.
    context: &'a P::Context,
}

impl<P> fmt::Display for DisplayWith<'_, P>
where
    P: Print + ?Sized,
{
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { target_value, context } = self;

        target_value.print_with_context(formatter, context)
    }
}

impl Print for () {
    type Context = ();

    #[inline]
    fn print_with_context<W>(&self, _: &mut W, _: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        Ok(())
    }
}

impl Print for bool {
    type Context = ();

    #[inline]
    fn print_with_context<W>(&self, writer: &mut W, _: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        write!(writer, "{self}")
    }
}

impl Print for str {
    type Context = ();

    #[inline]
    fn print_with_context<W>(&self, writer: &mut W, _: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        writer.write_str(self)
    }
}

impl Print for char {
    type Context = ();

    #[inline]
    fn print_with_context<W>(&self, writer: &mut W, _: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        writer.write_char(*self)
    }
}

impl<T> Print for &T
where
    T: Print + ?Sized,
{
    type Context = T::Context;

    #[inline]
    fn print_with_context<W>(&self, writer: &mut W, context: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        T::print_with_context(self, writer, context)
    }
}

impl<T> Print for [T]
where
    T: Print,
{
    type Context = T::Context;

    #[inline]
    fn print_with_context<W>(&self, writer: &mut W, context: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        writer.write_char('[')?;

        let mut first = true;

        for element in self {
            if first {
                first = false;
            } else {
                writer.write_str(", ")?;
            }

            element.print_with_context(writer, context)?;
        }

        writer.write_char(']')
    }
}

impl<const N: usize, T> Print for [T; N]
where
    T: Print,
{
    type Context = T::Context;

    #[inline]
    fn print_with_context<W>(&self, writer: &mut W, context: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        <[T] as Print>::print_with_context(self, writer, context)
    }
}

/// Prints [`Option::Some`] and emits no output for [`Option::None`].
impl<T> Print for Option<T>
where
    T: Print,
{
    type Context = T::Context;

    #[inline]
    fn print_with_context<W>(&self, writer: &mut W, context: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        match self {
            Some(value) => value.print_with_context(writer, context),
            None => Ok(()),
        }
    }
}

impl<T> Print for Range<T>
where
    T: Print,
{
    type Context = T::Context;

    #[inline]
    fn print_with_context<W>(&self, writer: &mut W, context: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        let Self { start, end } = self;

        start.print_with_context(writer, context)?;
        writer.write_str("..")?;
        end.print_with_context(writer, context)
    }
}

impl<T> Print for RangeInclusive<T>
where
    T: Print,
{
    type Context = T::Context;

    #[inline]
    fn print_with_context<W>(&self, writer: &mut W, context: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        self.start().print_with_context(writer, context)?;
        writer.write_str("..=")?;
        self.end().print_with_context(writer, context)
    }
}

impl<T> Print for RangeFrom<T>
where
    T: Print,
{
    type Context = T::Context;

    #[inline]
    fn print_with_context<W>(&self, writer: &mut W, context: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        let Self { start } = self;

        start.print_with_context(writer, context)?;
        writer.write_str("..")
    }
}

impl<T> Print for RangeTo<T>
where
    T: Print,
{
    type Context = T::Context;

    #[inline]
    fn print_with_context<W>(&self, writer: &mut W, context: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        let Self { end } = self;

        writer.write_str("..")?;
        end.print_with_context(writer, context)
    }
}

impl Print for RangeFull {
    type Context = ();

    #[inline]
    fn print_with_context<W>(&self, writer: &mut W, _: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        writer.write_str("..")
    }
}

impl Print for Infallible {
    type Context = ();

    #[inline]
    fn print_with_context<W>(&self, _: &mut W, _: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        match *self {}
    }
}

/// Implement [`Print`] for primitive numeric types.
macro_rules! impl_numeric {
    ($($target_type:ty),* $(,)?) => {
        $(
            impl Print for $target_type {
                type Context = ();

                #[inline]
                fn print_with_context<W>(&self, writer: &mut W, _: &Self::Context) -> fmt::Result
                where
                    W: fmt::Write,
                {
                    write!(writer, "{self}")
                }
            }
        )*
    };
}

/// Implement [`Print`] for non-zero numeric types.
macro_rules! impl_nonzero {
    ($($target_type:ty),* $(,)?) => {
        $(
            impl Print for core::num::NonZero<$target_type> {
                type Context = ();

                #[inline]
                fn print_with_context<W>(&self, writer: &mut W, _: &Self::Context) -> fmt::Result
                where
                    W: fmt::Write,
                {
                    write!(writer, "{}", self.get())
                }
            }
        )*
    };
}

impl_numeric!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64);
impl_nonzero!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

#[cfg(test)]
mod tests;
