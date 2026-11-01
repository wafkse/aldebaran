//! Severity and importance for diagnostics.
//!
//! See [`Severity`], [`Importance`], and [`Metadata`] for further information.

/// Severity describes how serious a diagnostic is to the user.
///
/// It does not decide control flow, so consumers choose how to act on each
/// level. The default is [`Error`](Self::Error).
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
pub enum Severity {
    /// A fatal (☠) error.
    ///
    /// Its exact meaning depends on the consumer, but a fatal error is generally
    /// understood to be so severe that it prevents the program from continuing,
    /// either forcibly or because of the nature of the error.
    Fatal,

    /// An error, but not world-ending.
    #[default]
    Error,

    /// A warning (non-fatal error).
    Warning,
}

/// Importance describes how prominently a diagnostic should be ordered when
/// a consumer combines several diagnostics into one list.
///
/// Ascending order places [`Critical`](Self::Critical) first and
/// [`Marginal`](Self::Marginal) last. The default is [`Trivial`](Self::Trivial).
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
pub enum Importance {
    /// Very important, so it sorts before the other importance levels.
    Critical,

    /// Somewhat important, between critical and marginal in the ordering.
    #[default]
    Trivial,

    /// Not important, but still noteworthy, so it sorts after the other levels.
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
    /// Select impact and priority explicitly without coupling the two classifications.
    #[inline]
    pub const fn tuple((severity, importance): (Severity, Importance)) -> Self {
        Self { severity, importance }
    }

    /// Warning severity with marginal importance.
    #[inline]
    pub const fn marginal() -> Self {
        Self::tuple((Severity::Warning, Importance::Marginal))
    }

    /// Error severity with trivial importance.
    #[doc(alias = "default")]
    #[inline]
    pub const fn whatever() -> Self {
        Self::tuple((Severity::Error, Importance::Trivial))
    }
}

impl Metadata {
    /// Select impact and priority explicitly without imposing a relationship between them.
    #[inline]
    pub const fn raw(severity: Severity, importance: Importance) -> Self {
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
