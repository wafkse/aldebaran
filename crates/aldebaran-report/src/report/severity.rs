//! Independent impact and priority classifications for diagnostic kinds.
//!
//! [`Metadata`] combines [`Severity`] and [`Importance`] so styling and consumer
//! policy can use structured values independently of a kind's displayed wording.

/// Descriptive severity classification for a diagnostic.
///
/// Severity does not determine control flow. Consumers decide how to act on it.
/// The default is [`Error`](Self::Error).
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
pub enum Severity {
    /// The relevant operation cannot continue, as determined by the consumer.
    Fatal,

    /// An error without the fatal classification.
    #[default]
    Error,

    /// A warning that leaves continuation policy to the consumer.
    Warning,
}

/// Diagnostic priority independently of severity.
///
/// Ascending ordering places [`Critical`](Self::Critical) first and
/// [`Marginal`](Self::Marginal) last. The default is [`Trivial`](Self::Trivial).
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
pub enum Importance {
    /// Highest priority, sorting before the other importance levels.
    Critical,

    /// Ordinary priority between critical and marginal diagnostics.
    #[default]
    Trivial,

    /// Lowest priority, sorting after the other importance levels.
    Marginal,
}

/// Severity and relative importance carried by a diagnostic kind.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[non_exhaustive]
pub struct Metadata {
    /// Severity used to classify the diagnostic's impact.
    pub severity: Severity,

    /// Relative importance used to prioritize the diagnostic.
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

    /// Impact classification used independently of relative priority.
    #[inline]
    pub const fn severity(&self) -> Severity {
        let &Self { severity, .. } = self;

        severity
    }

    /// Relative priority within the consumer's diagnostic policy.
    #[inline]
    pub const fn importance(&self) -> Importance {
        let &Self { importance, .. } = self;

        importance
    }

    /// Shared impact classification for metadata-aware consumers.
    #[inline]
    pub const fn severity_ref(&self) -> &Severity {
        let &Self { ref severity, .. } = self;

        severity
    }

    /// Shared priority classification, independently of impact.
    #[inline]
    pub const fn importance_ref(&self) -> &Importance {
        let &Self { ref importance, .. } = self;

        importance
    }

    /// Change the impact classification without changing relative priority.
    #[inline]
    pub const fn severity_mut(&mut self) -> &mut Severity {
        let &mut Self { ref mut severity, .. } = self;

        severity
    }

    /// Change relative priority without changing the impact classification.
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
