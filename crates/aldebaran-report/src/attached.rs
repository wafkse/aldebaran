//! Source association for diagnostics that store their locations separately.
//!
//! [`Attached`] exposes an existing [`Report`] as a [`SourceReport`] by borrowing
//! its source. The source and report may have different owners and lifetimes.

use aldebaran_source::prelude::{SourceLines, SourceMetadata};

use crate::report::{Report, SourceReport};

/// A report borrowed together with the source used for rendering.
///
/// The wrapper adds only the source relationship required by rendering. All
/// report semantics continue to delegate to the borrowed report, so attachment
/// does not duplicate annotations, titles, or diagnostic kinds.
///
/// Pairing checks neither source identity nor target bounds, so callers must
/// supply a source whose coordinates match the report's targets.
#[derive(Debug, Clone, Copy)]
// NOTE(invariant): None.
pub struct Attached<'source, 'report, S, R>
where
    S: SourceLines<'source> + SourceMetadata<'source> + ?Sized,
    R: Report + ?Sized,
{
    /// Concrete source used by renderers.
    source: &'source S,

    /// Source-independent report value.
    report: &'report R,
}

impl<'source, 'report, S, R> Attached<'source, 'report, S, R>
where
    S: SourceLines<'source> + SourceMetadata<'source> + ?Sized,
    R: Report + ?Sized,
{
    /// Interpret the report's existing targets against `source` without validating them.
    #[inline]
    #[must_use]
    pub const fn new(source: &'source S, report: &'report R) -> Self {
        Self { source, report }
    }

    /// Retrieve the rendering source.
    ///
    /// The returned reference keeps the source's original lifetime, independently of the report borrow.
    #[inline]
    #[must_use]
    pub const fn source(&self) -> &'source S {
        let &Self { source, .. } = self;

        source
    }

    /// Retrieve the source-independent report.
    #[inline]
    #[must_use]
    pub const fn report(&self) -> &'report R {
        let &Self { report, .. } = self;

        report
    }
}

impl<'source, 'report, S, R> Report for Attached<'source, 'report, S, R>
where
    S: SourceLines<'source> + SourceMetadata<'source> + ?Sized,
    R: Report + ?Sized,
{
    type Title = R::Title;

    type Kind = R::Kind;

    type Annotations = R::Annotations;

    #[inline]
    fn kind(&self) -> &Self::Kind {
        let Self { report, .. } = self;

        R::kind(report)
    }

    #[inline]
    fn title(&self) -> &Self::Title {
        let Self { report, .. } = self;

        R::title(report)
    }

    #[inline]
    fn annotations(&self) -> &Self::Annotations {
        let Self { report, .. } = self;

        R::annotations(report)
    }
}

impl<'source, 'report, S, R> SourceReport<'source> for Attached<'source, 'report, S, R>
where
    S: SourceLines<'source> + SourceMetadata<'source> + ?Sized,
    R: Report + ?Sized,
{
    type Source = S;

    #[inline]
    fn source(&self) -> &Self::Source {
        Attached::source(self)
    }
}
