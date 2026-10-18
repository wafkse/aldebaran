//! Generic emission of ANSI control sequences.
//!
//! [`AnsiSequence`] describes how one policy writes its control sequence to a
//! formatting sink for a supplied input. Color and style implementations use this
//! interface so sequence construction remains separate from higher-level painting.

use core::fmt;

/// A trait that represents a type that is capable of emitting ANSI sequences.
pub trait AnsiSequence<Input> {
    /// Emit the corresponding ANSI sequence, taking an input.
    fn emit_with_input<W>(&self, writer: &mut W, input: Input) -> fmt::Result
    where
        W: fmt::Write;

    /// Emit the corresponding ANSI sequence, using the default input.
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
