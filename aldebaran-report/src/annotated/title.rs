//! Printable titles attached to report and annotation structures.
//!
//! [`Title`] exposes a printable view while preserving the context required by
//! the underlying value. Textual renderers may choose to require a default
//! context, while context-aware renderers can provide one explicitly.

use aldebaran_print::prelude::Print;

/// The title of a message-carrying structure.
pub trait Title {
    /// External data required while printing this title.
    type Context;

    /// Determine the printable content of the title.
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
