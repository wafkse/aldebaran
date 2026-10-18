//! Terminal control sequences written to caller-provided formatting sinks.
//!
//! [`AnsiSequence`] separates a sequence's stored settings from per-emission
//! input such as the foreground or background surface. It emits bytes without
//! checking terminal capabilities or tracking previous terminal state.

use core::fmt;

/// Terminal settings that can be emitted with caller-selected input.
/// Implementations do not require a terminal connection and may emit to any formatting sink.
pub trait AnsiSequence<Input> {
    /// Emit settings with per-use input, such as the target color surface.
    /// A formatting error may leave a prefix of the sequence in the sink.
    fn emit_with_input<W>(&self, writer: &mut W, input: Input) -> fmt::Result
    where
        W: fmt::Write;

    /// Emit settings using the input type's default selection.
    /// For color surfaces this selects the foreground.
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
