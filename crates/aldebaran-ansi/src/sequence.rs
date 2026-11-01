//! Generic emission of ANSI control sequences.
//!
//! See [`AnsiSequence`] for further information.

use core::fmt;

/// A trait that represents a type that is capable of emitting ANSI sequences.
///
/// Implementations can write to any formatting sink and do not require a terminal connection.
pub trait AnsiSequence<Input> {
    /// Emit the corresponding ANSI sequence, taking an input. A formatting error
    /// may leave a prefix of the sequence in the sink.
    fn emit_with_input<W>(&self, writer: &mut W, input: Input) -> fmt::Result
    where
        W: fmt::Write;

    /// Emit the corresponding ANSI sequence, using the default input. For color
    /// surfaces the default selects the foreground.
    #[inline]
    fn emit<W>(&self, writer: &mut W) -> fmt::Result
    where
        W: fmt::Write,
        Input: Default,
    {
        self.emit_with_input(writer, Default::default())
    }
}

impl<T, I> AnsiSequence<I> for &T
where
    T: AnsiSequence<I>,
{
    #[inline]
    fn emit_with_input<W>(&self, writer: &mut W, input: I) -> fmt::Result
    where
        W: fmt::Write,
    {
        (*self).emit_with_input(writer, input)
    }
}
