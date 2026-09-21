//! One-shot reports with an explicit diagnostic kind.
//!
//! [`OneshotKinded`] combines a borrowed source, one annotation, and a caller
//! supplied error kind. It keeps kind policy explicit while avoiding a custom
//! report structure for short-lived diagnostics.

use aldebaran_dsa::prelude::RefOneOrMore;
use aldebaran_source::prelude::{SourceLines, SourceMetadata};

use crate::{
    annotated::Annotations,
    prelude::{Annotated, ErrorKind, Report},
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
    /// Create a new [`OneshotConcise`] error with the given source and
    /// annotation.
    ///
    /// [`OneshotConcise`]: super::OneshotConcise
    #[inline]
    pub const fn from_raw_parts(source: &'a S, annotation: A, kind: K) -> Self {
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

        &annotation
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

impl<'a, A, K, S> crate::report::SourceReport<'a> for OneshotKinded<'a, A, K, S>
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
    fn list(&self) -> Option<RefOneOrMore<'_, Self::Annotation>> {
        let Self { annotation, .. } = self;

        Some(RefOneOrMore::single(annotation))
    }
}
