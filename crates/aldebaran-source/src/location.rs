//! A module that provides a generic way to interface with source code
//! locations.
//!
//! In other words, this module wants to expose a trait that can be implemented
//! to represent an in-memory source, a file source, or any other kind of source
//! location in an error report.
//!
//! See the [`Location`] trait for more information.
//!
//! [`Location`] marks printable values such as file names. Its printing
//! context can resolve the origin without coupling that representation to
//! the source's component or span types.

use aldebaran_print::prelude::Print;

/// A [`Print`] value that identifies a source origin, such as a file or logical name.
pub trait Location: Print {}
