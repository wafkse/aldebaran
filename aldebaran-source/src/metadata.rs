//! Metadata attached to source representations.
//!
//! [`SourceMetadata`] separates source contents from contextual information such
//! as file or logical location. Sources without location knowledge use [`Unknown`]
//! so generic reporting code can retain one metadata capability boundary.

use core::fmt;

use aldebaran_print::prelude::Print;

use crate::source::Source;

use crate::location::Location;

/// A trait for sources that include metadata about themselves.
///
/// In most cases, this is used to express a source location of some kind.
#[diagnostic::on_unimplemented(label = "{Self} this source does not include metadata")]
pub trait SourceMetadata<'a>: Source<'a> {
    /// The metadata type for this source.
    type Metadata;

    /// Retrieve the metadata for this source.
    fn metadata(&self) -> &Self::Metadata;
}

/// A trait for metadata types that are attached to sources.
pub trait Metadata {}

/// A metadata type that effectively acts as an unknown location.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Unknown;

impl Metadata for Unknown {}

impl Location for Unknown {}

impl Print for Unknown {
    type Context = ();
    #[inline]
    fn print_with_ctx<W>(&self, writer: &mut W, _: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        write!(writer, "<unknown>")
    }
}
