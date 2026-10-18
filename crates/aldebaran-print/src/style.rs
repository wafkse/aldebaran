//! Presentation styles for formatted output.
//!
//! [`Stylus`] lets a writer apply presentation while a closure writes. The
//! presentation may appear in the output or be held as state by the writer.

use core::fmt;

/// Presentation surrounding a closure's output to a [`fmt::Write`] sink.
///
/// Styles may write control sequences or change writer-owned state. The
/// associated error describes presentation failures independently of errors
/// produced by the output closure.
pub trait Stylus<W>
where
    W: fmt::Write,
{
    /// Errors from applying or removing presentation state. Implementations
    /// whose setup and cleanup cannot fail use
    /// [`Infallible`](core::convert::Infallible).
    type Error;

    /// Apply style around the output written by `closure`.
    ///
    /// The outer result reports style-operation failures, while the inner
    /// result is returned by the closure. Cleanup can fail after output has
    /// been written, and its failure may take precedence over the closure's
    /// result. Implementations may or may not run cleanup if the closure panics.
    fn style<F, O, E>(&self, writer: &mut W, closure: F) -> Result<Result<O, E>, Self::Error>
    where
        F: FnOnce(&mut W) -> Result<O, E>;
}
