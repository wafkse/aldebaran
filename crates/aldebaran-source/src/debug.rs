//! Debug formatting requirements for sources and their components.
//!
//! [`SourceDebug`] lets generic debug output inspect both the source value and
//! individual traversal components without requiring these capabilities of every
//! [`Source`].

use core::fmt;

use crate::source::Source;

/// A trait for those [`Source`] types that can be debugged.
pub trait SourceDebug<'a>: Source<'a>
where
    Self: fmt::Debug,
    Self::Component: fmt::Debug,
{
}

/// Blanket implementation for all [`Source`] types that are also [`fmt::Debug`].
impl<'a, S> SourceDebug<'a> for S
where
    S: Source<'a> + fmt::Debug + ?Sized,
    Self::Component: fmt::Debug,
{
}
