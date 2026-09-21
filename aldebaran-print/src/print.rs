//! Context-aware printing into formatting sinks.
//!
//! [`Print`] is the small formatting contract used by Aldebaran types that need
//! sink-oriented output without committing to [`core::fmt::Display`]. A value
//! may require external contextual data such as an interner or arena while
//! printing. Values with a default context retain the direct [`Print::print`]
//! convenience path.

use core::{
    convert::Infallible,
    fmt,
    ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo},
};

/// A value that can write its textual representation into a formatting sink.
#[diagnostic::on_unimplemented(message = "{Self} is not printable")]
pub trait Print {
    /// External data required while printing this value.
    type Context;

    /// Write this value using a default context.
    ///
    /// This convenience operation is available only when the context has a
    /// natural default value.
    #[inline]
    fn print<W>(&self, writer: &mut W) -> fmt::Result
    where
        W: fmt::Write,
        Self::Context: Default,
    {
        let context = Self::Context::default();

        self.print_with_ctx(writer, &context)
    }

    /// Write this value using `context`.
    fn print_with_ctx<W>(&self, writer: &mut W, context: &Self::Context) -> fmt::Result
    where
        W: fmt::Write;

    /// Borrow this value as a standard display value using a default context.
    #[inline]
    #[must_use]
    fn display(&self) -> impl fmt::Display + '_
    where
        Self::Context: Default,
    {
        DisplayDefault(self)
    }

    /// Borrow this value as a standard display value using `context`.
    #[inline]
    #[must_use]
    fn display_with_ctx<'a>(&'a self, context: &'a Self::Context) -> impl fmt::Display + 'a {
        DisplayWith {
            target_value: self,
            context,
        }
    }
}

/// Borrowed standard-display view of one default-context [`Print`] value.
#[repr(transparent)]
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

/// Borrowed standard-display view of one contextual [`Print`] value.
struct DisplayWith<'a, P>
where
    P: Print + ?Sized,
{
    target_value: &'a P,
    context: &'a P::Context,
}

impl<P> fmt::Display for DisplayWith<'_, P>
where
    P: Print + ?Sized,
{
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { target_value, context } = self;

        target_value.print_with_ctx(formatter, context)
    }
}

impl Print for () {
    type Context = ();

    #[inline]
    fn print_with_ctx<W>(&self, _: &mut W, _: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        Ok(())
    }
}

impl Print for bool {
    type Context = ();

    #[inline]
    fn print_with_ctx<W>(&self, writer: &mut W, _: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        write!(writer, "{self}")
    }
}

impl Print for str {
    type Context = ();

    #[inline]
    fn print_with_ctx<W>(&self, writer: &mut W, _: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        writer.write_str(self)
    }
}

impl Print for char {
    type Context = ();

    #[inline]
    fn print_with_ctx<W>(&self, writer: &mut W, _: &Self::Context) -> fmt::Result
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
    fn print_with_ctx<W>(&self, writer: &mut W, context: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        T::print_with_ctx(self, writer, context)
    }
}

impl<T> Print for [T]
where
    T: Print,
{
    type Context = T::Context;

    #[inline]
    fn print_with_ctx<W>(&self, writer: &mut W, context: &Self::Context) -> fmt::Result
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

            element.print_with_ctx(writer, context)?;
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
    fn print_with_ctx<W>(&self, writer: &mut W, context: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        <[T] as Print>::print_with_ctx(self, writer, context)
    }
}

impl<T> Print for Option<T>
where
    T: Print,
{
    type Context = T::Context;

    #[inline]
    fn print_with_ctx<W>(&self, writer: &mut W, context: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        match self {
            Some(value) => value.print_with_ctx(writer, context),
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
    fn print_with_ctx<W>(&self, writer: &mut W, context: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        let Self { start, end } = self;

        start.print_with_ctx(writer, context)?;
        writer.write_str("..")?;
        end.print_with_ctx(writer, context)
    }
}

impl<T> Print for RangeInclusive<T>
where
    T: Print,
{
    type Context = T::Context;

    #[inline]
    fn print_with_ctx<W>(&self, writer: &mut W, context: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        self.start().print_with_ctx(writer, context)?;
        writer.write_str("..=")?;
        self.end().print_with_ctx(writer, context)
    }
}

impl<T> Print for RangeFrom<T>
where
    T: Print,
{
    type Context = T::Context;

    #[inline]
    fn print_with_ctx<W>(&self, writer: &mut W, context: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        let Self { start } = self;

        start.print_with_ctx(writer, context)?;
        writer.write_str("..")
    }
}

impl<T> Print for RangeTo<T>
where
    T: Print,
{
    type Context = T::Context;

    #[inline]
    fn print_with_ctx<W>(&self, writer: &mut W, context: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        let Self { end } = self;

        writer.write_str("..")?;
        end.print_with_ctx(writer, context)
    }
}

impl Print for RangeFull {
    type Context = ();

    #[inline]
    fn print_with_ctx<W>(&self, writer: &mut W, _: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        writer.write_str("..")
    }
}

impl Print for Infallible {
    type Context = ();

    #[inline]
    fn print_with_ctx<W>(&self, _: &mut W, _: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        match *self {}
    }
}

macro_rules! impl_numeric {
    ($($target_type:ty),* $(,)?) => {
        $(
            impl Print for $target_type {
                type Context = ();

                #[inline]
                fn print_with_ctx<W>(&self, writer: &mut W, _: &Self::Context) -> fmt::Result
                where
                    W: fmt::Write,
                {
                    write!(writer, "{self}")
                }
            }
        )*
    };
}

macro_rules! impl_nonzero {
    ($($target_type:ty),* $(,)?) => {
        $(
            impl Print for core::num::NonZero<$target_type> {
                type Context = ();

                #[inline]
                fn print_with_ctx<W>(&self, writer: &mut W, _: &Self::Context) -> fmt::Result
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
mod tests {
    extern crate alloc;

    use alloc::{format, string::String};

    use crate::combine::Combine as _;

    use super::Print as _;

    struct Names(&'static [&'static str]);

    struct Name(usize);

    impl super::Print for Name {
        type Context = Names;

        fn print_with_ctx<W>(&self, writer: &mut W, context: &Self::Context) -> core::fmt::Result
        where
            W: core::fmt::Write,
        {
            let Self(target_index) = self;
            let Names(target_names) = context;

            writer.write_str(target_names[*target_index])
        }
    }

    #[test]
    fn composition_writes_directly_in_order() {
        let mut output = String::new();

        "ab".sequence('-').sequence('x'.times(3)).print(&mut output).unwrap();

        assert_eq!(output, "ab-xxx");
    }

    #[test]
    fn contextual_values_do_not_require_default_contexts() {
        let mut output = String::new();
        let context = Names(&["left", "right"]);

        Name(0).aggregate(Name(1)).print_with_ctx(&mut output, &context).unwrap();

        assert_eq!(output, "leftright");
    }

    #[test]
    fn default_context_values_can_use_standard_display_formatting() {
        let output = format!("value={}", 42_u8.display());

        assert_eq!(output, "value=42");
    }

    #[test]
    fn contextual_values_can_use_standard_display_formatting() {
        let context = Names(&["left", "right"]);
        let target_name = Name(1);

        let output = format!("name={}", target_name.display_with_ctx(&context));

        assert_eq!(output, "name=right");
    }

    #[test]
    fn slices_keep_delimiter_order() {
        let mut output = String::new();

        [1_u8, 2, 3].print(&mut output).unwrap();

        assert_eq!(output, "[1, 2, 3]");
    }
}
