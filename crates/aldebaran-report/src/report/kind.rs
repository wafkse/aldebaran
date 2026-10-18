//! Diagnostic classifications with independent wording and styling metadata.
//!
//! [`ErrorKind`] supplies the printed descriptor and uses [`Connotate`] to expose
//! severity and importance. [`Simple`] provides common classifications whose
//! wording remains distinct even when their metadata is the same.

use core::fmt;

use aldebaran_print::prelude::Print;

use crate::report::{connotate::Connotate, severity::Metadata};

/// A diagnostic classification with printable wording and semantic metadata.
///
/// Renderers can use the descriptor for text and the [`Connotate`] result for
/// styling or ordering without deriving meaning from the displayed word.
pub trait ErrorKind: Connotate<Subject = Metadata> {
    /// Printable descriptor associated with this classification.
    type Descriptor: Print;

    /// Borrow the printable descriptor for this classification.
    fn descriptor(&self) -> &Self::Descriptor;
}

/// Common diagnostic classifications with fixed wording and metadata.
///
/// [`Warning`](Self::Warning) and [`Lint`](Self::Lint) share styling metadata but
/// print different descriptors. The default is [`Error`](Self::Error).
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Default, Clone, Copy)]
pub enum Simple {
    /// Prints `error` with error severity and trivial importance.
    #[default]
    Error,

    /// Prints `warning` with warning severity and marginal importance.
    Warning,

    /// Prints `lint` with the same metadata as [`Warning`](Self::Warning).
    Lint,
}

impl ErrorKind for Simple {
    type Descriptor = Self;

    #[inline]
    fn descriptor(&self) -> &Self::Descriptor {
        self
    }
}

impl Print for Simple {
    type Context = ();
    #[inline]
    fn print_with_context<W>(&self, writer: &mut W, _: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        let target_str = match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Lint => "lint",
        };

        target_str.print(writer)
    }
}

impl Connotate for Simple {
    type Subject = Metadata;

    #[inline]
    fn connotates(&self) -> Self::Subject {
        match self {
            Self::Error => Metadata::whatever(),
            Self::Warning | Self::Lint => Metadata::marginal(),
        }
    }
}
