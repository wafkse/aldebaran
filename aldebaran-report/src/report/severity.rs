//! Semantic severity and importance metadata for diagnostics.
//!
//! [`Severity`] captures the diagnostic class presented to a user. [`Importance`]
//! captures ordering significance independently. [`Metadata`] combines those
//! dimensions so renderers and aggregators can reason about diagnostics structurally.

/// A severity level.
///
/// This is used to expose the intricacies of in a more detailed manner.
///
/// Note that the [`default`](Default) severity is deemed a (non-fatal)
/// [`error`](Severity::Error).
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
pub enum Severity {
    /// A fatal (☠) error.
    ///
    /// Albeit the semantics of this are not strictly defined, it is generally
    /// understood that a fatal error is an error that is so severe that it
    /// prohibits the program from continuing, forcibly or due to the nature of
    /// the error itself.
    Fatal,

    /// An error, but not world-ending.
    #[default]
    Error,

    /// A warning (non-fatal error).
    Warning,
}

/// An importance level.
///
/// This is used to determine the *intrinsic ordering* of errors, that is, how
/// to order them based on their importance if coalescing them into a single
/// list is necessary.
///
/// Note that the [`default`](Default) importance is
/// [`trivial`](Importance::Trivial).
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
pub enum Importance {
    /// Very important. At the foremost level.
    Critical,

    /// Somewhat important. May be at the forefront, but not always.
    #[default]
    Trivial,

    /// Not important, albeit still noteworthy. At the backmost level.
    Marginal,
}

/// A piece of error-specific metadata.
///
/// This is used to provide additional information about an error, such as its
/// severity or overall importance.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[non_exhaustive]
pub struct Metadata {
    /// The associated severity of the error.
    pub severity: Severity,

    /// The intrinsic importance of the error.
    pub importance: Importance,
}

impl Metadata {
    /// Create a new [`Metadata`] from a 2-tuple composed of [`Severity`] and
    /// [`Importance`] levels.
    #[inline]
    pub const fn tuple((severity, importance): (Severity, Importance)) -> Self {
        Self { severity, importance }
    }

    /// A [`Metadata`] with the least severity and importance.
    ///
    /// This is provided as a shortcut to create warnings and lints with ease.
    #[inline]
    pub const fn marginal() -> Self {
        Self::tuple((Severity::Warning, Importance::Marginal))
    }

    /// A [`Metadata`] with the default severity and importance.
    #[doc(alias = "default")]
    #[inline]
    pub const fn whatever() -> Self {
        Self::tuple((Severity::Error, Importance::Trivial))
    }
}

impl Metadata {
    /// Create a new [`Metadata`] from its raw parts: a [`Severity`] and
    /// [`Importance`] levels.
    #[inline]
    pub const fn from_raw_parts(severity: Severity, importance: Importance) -> Self {
        Self { severity, importance }
    }

    /// Retrieve the associated severity of this error metadata.
    #[inline]
    pub const fn severity(&self) -> Severity {
        let &Self { severity, .. } = self;

        severity
    }

    /// Retrieve the intrinsic importance of this error metadata.
    #[inline]
    pub const fn importance(&self) -> Importance {
        let &Self { importance, .. } = self;

        importance
    }

    /// Retrieve *(an immutable reference)* to the associated severity of this
    /// metadata.
    #[inline]
    pub const fn severity_ref(&self) -> &Severity {
        let &Self { ref severity, .. } = self;

        severity
    }

    /// Retrieve *(an immutable reference)* to the intrinsic importance of this
    /// metadata.
    #[inline]
    pub const fn importance_ref(&self) -> &Importance {
        let &Self { ref importance, .. } = self;

        importance
    }

    /// Retrieve *(a mutable reference)* to the associated severity of this
    /// metadata.
    #[inline]
    pub const fn severity_mut(&mut self) -> &mut Severity {
        let &mut Self { ref mut severity, .. } = self;

        severity
    }

    /// Retrieve *(a mutable reference)* to the intrinsic importance of this
    /// metadata.
    #[inline]
    pub const fn importance_mut(&mut self) -> &mut Importance {
        let &mut Self { ref mut importance, .. } = self;

        importance
    }
}

impl Default for Metadata {
    #[inline]
    fn default() -> Self {
        Self::whatever()
    }
}
