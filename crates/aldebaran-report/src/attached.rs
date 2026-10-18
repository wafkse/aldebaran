//! Source association for diagnostics that store their locations separately.
//!
//! [`Attached`] exposes an existing [`Report`] as a [`SourceReport`] by borrowing
//! its source. The source and report may have different owners and lifetimes.

use aldebaran_source::prelude::{SourceLines, SourceMetadata};

use crate::report::{Report, SourceReport};

/// A diagnostic and its source borrowed without copying annotation storage.
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
    /// Source whose coordinates interpret the report's annotation targets.
    source: &'source S,

    /// Diagnostic content retained by its original owner.
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

    /// Rendering source with its original lifetime, independent of the report borrow.
    #[inline]
    #[must_use]
    pub const fn source(&self) -> &'source S {
        let &Self { source, .. } = self;

        source
    }

    /// Original diagnostic value, shared without copying its annotation storage.
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
