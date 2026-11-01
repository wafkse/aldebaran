//! A module that provides a generic way to interface with source code
//! locations.
//!
//! See [`Location`] for further information.

use aldebaran_print::prelude::Print;

/// An abstract source code location.
///
/// This can be a myriad of things, but, in general, it should represent a
/// location where a source code snippet is procedent from. Its [`Print`]
/// context can resolve the origin without coupling it to the source's
/// component or span types.
pub trait Location: Print {}
