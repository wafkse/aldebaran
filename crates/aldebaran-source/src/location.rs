//! Source origins rendered separately from content coordinates.
//!
//! [`Location`] marks printable values such as file names. Its printing
//! context can resolve the origin without coupling that representation to
//! the source's component or span types.

use aldebaran_print::prelude::Print;

/// A [`Print`] value that identifies a source origin, such as a file or logical name.
pub trait Location: Print {}
