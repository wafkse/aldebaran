//! Printable titles attached to report and annotation structures.
//!
//! See [`Title`] for further information.
//!
//! Textual renderers may require a default context, while context-aware renderers can
//! supply one explicitly.

use aldebaran_print::prelude::Print;

/// Printable content for a diagnostic heading or annotation message.
///
/// Values implementing [`Print`] already satisfy this contract. A custom title
/// can instead supply a printable view that uses its own external context.
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
