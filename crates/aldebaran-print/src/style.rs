//! Scoped application of presentation styles to formatting sinks.
//!
//! [`Stylus`] wraps one write operation between style setup and teardown. The
//! writer remains generic so in-band ANSI styling and out-of-band presentation
//! state can implement the same semantic style boundary.

use core::fmt;

/// A stylus for a generic [`writer`](fmt::Write) `W`.
///
/// The rationale behind being generic over a writer is to allow for the case
/// where stylistic settings require interacting with the writer in another
/// manner, this extraneous interaction can be expressed by additional trait
/// bounds, or a concrete type.
///
/// A motivating example of such occurence is the difference between a writer
/// that incorporates style in the flow of data itself compared to a stream that
/// handles style through flags but is indifferent to the stream itself.
///
/// This architecture allows for both cases to coexist, and for a single type to
/// handle different types of writers.
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
    /// writers where the style itself can **linger** after it has been
    /// applied for an undefined amount of time (which is the case for
    /// ANSI-compatible writers), to combat this, style is applied in an
    /// initial stage, and a teardown stage (optional), where it can allow
    /// for the style to be removed.
    fn style<F, O, E>(&self, writer: &mut W, closure: F) -> Result<Result<O, E>, Self::Error>
    where
        F: FnOnce(&mut W) -> Result<O, E>;
}
