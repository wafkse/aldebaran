//! Module providing the correspondence between a source and its owned form.
//!
//! See [`SourceOwned`] for more information.

use crate::source::Source;

use alloc::borrow::ToOwned;

/// A trait for those sources that can be transformed into their owned form.
///
/// This is purely an extension of the [`Source`] trait.
pub trait SourceOwned<'source>: Source<'source> + ToOwned {}

impl<'source, S> SourceOwned<'source> for S where S: Source<'source> + ToOwned + ?Sized {}
