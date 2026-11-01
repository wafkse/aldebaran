//! Scoped application of presentation styles to formatting sinks.
//!
//! See [`Stylus`] for further information.

use core::fmt;

/// A stylus for a generic [`writer`](fmt::Write) `W`.
///
/// The writer is generic so styling can either be written into the output or
/// handled through state kept by the writer. Additional writer requirements can
/// be expressed through trait bounds or a concrete type. One style can therefore
/// work with different kinds of writers.
///
/// Setup and teardown failures are reported separately from the closure result.
/// Cleanup can fail after output has been written, and implementations may or may
/// not run cleanup if the closure panics.
pub trait Stylus<W>
where
    W: fmt::Write,
{
    /// The error type (or [`Infallible`](core::convert::Infallible) if the
    /// operation is infallible) for this [`Stylus`].
    type Error;

    /// Apply the implementation-defined style in a 2-stage manner.
    ///
    /// The closure `F` will be invoked when the stylistic settings have been
    /// applied to the target writer.
    ///
    /// The 2-stage methodology is used to allow for the possibility to style
    /// writers where the style itself can **linger** after it has been applied
    /// for an undefined amount of time (as with ANSI-compatible writers). The
    /// setup stage is followed by teardown, which allows the style to be removed.
    /// The outer result reports failures while applying or removing the style,
    /// so a cleanup failure may take precedence over the closure's result.
    /// Whether cleanup runs after a panic depends on the implementation.
    fn style<F, O, E>(&self, writer: &mut W, closure: F) -> Result<Result<O, E>, Self::Error>
    where
        F: FnOnce(&mut W) -> Result<O, E>;
}
