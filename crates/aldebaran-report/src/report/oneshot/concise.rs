//! Compact reports for diagnostics with a single annotated location.
//!
//! The annotation can provide the wording shown as the report title.

use aldebaran_baseline::prelude::RefNonempty;
use aldebaran_source::prelude::{SourceLines, SourceMetadata};

use crate::{
    annotated::Annotations,
    prelude::{Annotated, Report, Simple},
    report::SourceReport,
};

/// A one-shot diagnostic whose sole annotation also provides its title.
///
/// Its diagnostic kind is [`Simple::Error`].
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
// NOTE(invariant): None.
pub struct OneshotConcise<'source, A, S = str>
where
    A: Annotated,
    S: SourceLines<'source> + SourceMetadata<'source> + ?Sized,
{
    /// Source whose coordinates interpret the annotation target.
    source: &'source S,

    /// Message and target reused as the report heading and primary location.
    annotation: A,
}

impl<'source, A, S> OneshotConcise<'source, A, S>
where
    A: Annotated,
    S: SourceLines<'source> + SourceMetadata<'source> + ?Sized,
{
    /// Use the annotation message as the report heading under the error classification.
    #[inline]
    pub const fn raw(source: &'source S, annotation: A) -> Self {
        Self { source, annotation }
    }

    /// Source against which the primary annotation target is interpreted.
    #[inline]
    pub const fn source(&self) -> &'source S {
        let &Self { source, .. } = self;

        source
    }

    /// Annotation supplying the report heading as well as its primary location.
    #[inline]
    pub const fn annotation(&self) -> &A {
        let &Self { ref annotation, .. } = self;

        annotation
    }
}

impl<'source, A, S> Report for OneshotConcise<'source, A, S>
where
    A: Annotated,
    S: SourceLines<'source> + SourceMetadata<'source> + ?Sized,
{
    type Title = A::Title;

    type Kind = Simple;

    type Annotations = Self;

    #[inline]
    fn title(&self) -> &Self::Title {
        let Self { annotation, .. } = self;

        annotation.message()
    }

    #[inline]
    fn kind(&self) -> &Self::Kind {
        &Simple::Error
    }

    #[inline]
    fn annotations(&self) -> &Self::Annotations {
        self
    }
}

impl<'source, A, S> SourceReport<'source> for OneshotConcise<'source, A, S>
where
    A: Annotated,
    S: SourceLines<'source> + SourceMetadata<'source> + ?Sized,
{
    type Source = S;

    #[inline]
    fn source(&self) -> &Self::Source {
        let &Self { source, .. } = self;

        source
    }
}

impl<'source, A, S> Annotations for OneshotConcise<'source, A, S>
where
    A: Annotated,
    S: SourceLines<'source> + SourceMetadata<'source> + ?Sized,
{
    type Annotation = A;

    #[inline]
    fn list(&self) -> Option<RefNonempty<'_, Self::Annotation>> {
        let Self { annotation, .. } = self;

        Some(RefNonempty::single(annotation))
    }
}
