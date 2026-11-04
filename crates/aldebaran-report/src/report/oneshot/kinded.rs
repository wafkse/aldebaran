//! Compact reports for diagnostics that need a nondefault classification.
//!
//! See [`OneshotKinded`] for further information.

use aldebaran_baseline::prelude::RefNonempty;
use aldebaran_source::prelude::{SourceLines, SourceMetadata};

use crate::{
    annotated::Annotations,
    prelude::{Annotated, ErrorKind, Report},
    report::SourceReport,
};

/// A diagnostic whose annotation supplies its heading and sole source location.
///
/// Its message can be reused under different diagnostic classifications because
/// the [`ErrorKind`] is independent of the annotation.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
// NOTE(invariant): None.
pub struct OneshotKinded<'a, A, K, S = str>
where
    A: Annotated,
    S: SourceLines<'a> + SourceMetadata<'a> + ?Sized,
    K: ErrorKind,
{
    /// The source of this error.
    source: &'a S,

    /// The singular annotation of this error.
    annotation: A,

    /// The kind of this error.
    kind: K,
}

impl<'a, A, K, S> OneshotKinded<'a, A, K, S>
where
    A: Annotated,
    S: SourceLines<'a> + SourceMetadata<'a> + ?Sized,
    K: ErrorKind,
{
    /// Create a new [`OneshotKinded`] error with the given source, annotation,
    /// and kind.
    #[inline]
    pub const fn raw(source: &'a S, annotation: A, kind: K) -> Self {
        Self { source, annotation, kind }
    }

    /// Retrieve the kind of this error.
    #[inline]
    pub const fn kind(&self) -> &K {
        let &Self { ref kind, .. } = self;

        kind
    }

    /// Retrieve the source of this error.
    #[inline]
    pub const fn source(&self) -> &'a S {
        let &Self { source, .. } = self;

        source
    }

    /// Retrieve the annotation of this error.
    #[inline]
    pub const fn annotation(&self) -> &A {
        let &Self { ref annotation, .. } = self;

        annotation
    }
}

impl<'a, A, K, S> Report for OneshotKinded<'a, A, K, S>
where
    A: Annotated,
    S: SourceLines<'a> + SourceMetadata<'a> + ?Sized,
    K: ErrorKind,
{
    type Title = A::Title;

    type Kind = K;

    type Annotations = Self;

    #[inline]
    fn title(&self) -> &Self::Title {
        let Self { annotation, .. } = self;

        annotation.message()
    }

    #[inline]
    fn kind(&self) -> &Self::Kind {
        let Self { kind, .. } = self;

        kind
    }

    #[inline]
    fn annotations(&self) -> &Self::Annotations {
        self
    }
}

impl<'a, A, K, S> SourceReport<'a> for OneshotKinded<'a, A, K, S>
where
    A: Annotated,
    S: SourceLines<'a> + SourceMetadata<'a> + ?Sized,
    K: ErrorKind,
{
    type Source = S;

    #[inline]
    fn source(&self) -> &Self::Source {
        let &Self { source, .. } = self;

        source
    }
}

impl<'a, A, K, S> Annotations for OneshotKinded<'a, A, K, S>
where
    A: Annotated,
    S: SourceLines<'a> + SourceMetadata<'a> + ?Sized,
    K: ErrorKind,
{
    type Annotation = A;

    #[inline]
    fn list(&self) -> Option<RefNonempty<'_, Self::Annotation>> {
        let Self { annotation, .. } = self;

        Some(RefNonempty::single(annotation))
    }
}
