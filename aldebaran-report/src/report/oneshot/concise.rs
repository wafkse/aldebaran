//! Concise one-shot reports with a statically known primary annotation.
//!
//! [`OneshotConcise`] borrows a source and one annotation while deriving the
//! report title directly from that annotation. It avoids constructing a dedicated
//! error type when source, title, and target are already available together.

use aldebaran_dsa::prelude::RefOneOrMore;
use aldebaran_source::prelude::{SourceLines, SourceMetadata};

use crate::{
    annotated::Annotations,
    prelude::{Annotated, Report, Simple},
};

/// A simple, concise one-shot error type.
///
/// This contains a static, known amount of annotations, and is meant to be used
/// in contexts where a full-blown error type is not necessary.
///
/// This is a more concise version of the [`Oneshot`] error type, and is meant
/// to be used in contexts where both title and annotation are known beforehand.
///
/// [`Oneshot`]: super::Oneshot
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct OneshotConcise<'source, A, S = str>
where
    A: Annotated,
    S: SourceLines<'source> + SourceMetadata<'source> + ?Sized,
{
    /// The source of this error.
    source: &'source S,
    /// The singular annotation of this error.
    annotation: A,
}

impl<'source, A, S> OneshotConcise<'source, A, S>
where
    A: Annotated,
    S: SourceLines<'source> + SourceMetadata<'source> + ?Sized,
{
    /// Create a new [`OneshotConcise`] error with the given source and
    /// annotation.
    #[inline]
    pub const fn from_raw_parts(source: &'source S, annotation: A) -> Self {
        Self { source, annotation }
    }

    /// Retrieve the source of this error.
    #[inline]
    pub const fn source(&self) -> &'source S {
        let &Self { source, .. } = self;

        source
    }

    /// Retrieve the annotation of this error.
    #[inline]
    pub const fn annotation(&self) -> &A {
        let &Self { ref annotation, .. } = self;

        &annotation
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

impl<'source, A, S> crate::report::SourceReport<'source> for OneshotConcise<'source, A, S>
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
    fn list(&self) -> Option<RefOneOrMore<'_, Self::Annotation>> {
        let Self { annotation, .. } = self;

        Some(RefOneOrMore::single(annotation))
    }
}
