//! A module that provides a generic way to interface with source code
//! locations.
//!
//! In other words, this module wants to expose a trait that can be implemented
//! to represent an in-memory source, a file source, or any other kind of source
//! location in an error report.
//!
//! See the [`Location`] trait for more information.

use aldebaran_print::prelude::Print;

/// An abstract source code location.
///
/// This can be a myriad of things, but, in general, it should represent a
/// location where a source code snippet is procedent from.
pub trait Location: Print {}
