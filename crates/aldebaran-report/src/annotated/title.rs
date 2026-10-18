//! Printable titles attached to report and annotation structures.
//!
//! [`Title`] exposes a printable view while preserving the context required by
//! the underlying value. Textual renderers may choose to require a default
//! context, while context-aware renderers can provide it explicitly.

use aldebaran_print::prelude::Print;

/// Printable content for a diagnostic heading or annotation message.
///
/// Values implementing [`Print`] already satisfy this contract. A custom title
/// can instead supply a printable view that uses its own external context.
pub trait Title {
    /// Data the caller must supply when formatting the printable content.
    type Context;

    /// Printable view that interprets the context defined by this title.
    fn content(&self) -> impl Print<Context = Self::Context>;
}

impl<T> Title for T
where
    T: Print + ?Sized,
{
    type Context = T::Context;

    #[inline]
    fn content(&self) -> impl Print<Context = Self::Context> {
        self
    }
}
