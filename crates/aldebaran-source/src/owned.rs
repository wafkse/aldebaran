//! Owned conversion for sources retained by storage.
//!
//! [`SourceOwned`] connects a borrowed source representation with [`ToOwned`],
//! allowing storage consumers to select its associated owned form.

use crate::source::Source;

use alloc::borrow::ToOwned;

/// Source data that storage can retain through [`ToOwned::to_owned`].
pub trait SourceOwned<'source>: Source<'source> + ToOwned {}

impl<'source, S> SourceOwned<'source> for S where S: Source<'source> + ToOwned + ?Sized {}
