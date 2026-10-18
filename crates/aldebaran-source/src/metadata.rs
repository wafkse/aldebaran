//! Contextual metadata associated with source representations.
//!
//! [`SourceMetadata`] keeps source contents separate from context such as a file
//! or logical origin. [`Unknown`] supplies a printable placeholder when no such
//! context is available.

use core::fmt;

use aldebaran_print::prelude::Print;

use crate::source::Source;

use crate::location::Location;

/// A source that carries context alongside its contents.
#[diagnostic::on_unimplemented(label = "{Self} this source does not include metadata")]
pub trait SourceMetadata<'a>: Source<'a> {
    /// Origin or other source context retained independently of the content.
    type Metadata;

    /// Context used to identify this source independently of a selected span.
    fn metadata(&self) -> &Self::Metadata;
}

/// Classification of source-context values.
/// It imposes no formatting or source-coordinate requirements.
pub trait Metadata {}

/// A source context that prints as `<unknown>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Unknown;

impl Metadata for Unknown {}

impl Location for Unknown {}

impl Print for Unknown {
    type Context = ();
    #[inline]
    fn print_with_context<W>(&self, writer: &mut W, _: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        write!(writer, "<unknown>")
    }
}
