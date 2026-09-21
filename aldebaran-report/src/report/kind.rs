//! Structured diagnostic kinds and their semantic metadata.
//!
//! [`ErrorKind`] combines printable presentation with semantic connotation.
//! [`Simple`] provides the common error and warning categories while retaining a
//! metadata projection that renderers can inspect independently of wording.

use core::fmt;

use aldebaran_print::prelude::Print;

use crate::report::{connotate::Connotate, severity::Metadata};

/// A super-trait for error kinds.
///
/// This trait combines both the [`Print`] and the [`Connotate`] traits
/// into a single atomic unit.
pub trait ErrorKind: Connotate<Subject = Metadata> {
    /// The type representing the descriptor of the error kind.
    ///
    /// This serves as a human-readable representation of the error kind.
    type Descriptor: Print;

    /// Retrieve the [`descriptor`](Self::Descriptor) for this error kind.
    fn descriptor(&self) -> &Self::Descriptor;
}

/// Keep it simple, stupid.
///
/// This is used to determine the nature of the error, in other words, whether
/// it is an error, warning, lint, etc.
///
/// This enum is kept simplistic on purpose, as higher-level connotations are
/// not necessary for the error report.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Default, Clone, Copy)]
pub enum Simple {
    /// An error that is deemed non-fatal. The default kind.
    #[default]
    Error,

    /// A warning.
    Warning,

    /// A lint, or a suggestion.
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
    fn print_with_ctx<W>(&self, writer: &mut W, _: &Self::Context) -> fmt::Result
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
