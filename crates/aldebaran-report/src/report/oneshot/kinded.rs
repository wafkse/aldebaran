//! Compact reports for diagnostics that need a nondefault classification.
//!
//! The caller supplies the kind used by the report.

use aldebaran_baseline::prelude::RefOneOrMore;
use aldebaran_source::prelude::{SourceLines, SourceMetadata};

use crate::{
    annotated::Annotations,
    prelude::{Annotated, ErrorKind, Report},
    report::SourceReport,
};

/// A diagnostic whose annotation supplies its heading and sole source location.
///
/// The caller supplies the [`ErrorKind`] independently, so the annotation message
/// can be reused under different diagnostic classifications.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
// NOTE(invariant): None.
pub struct OneshotKinded<'a, A, K, S = str>
where
    A: Annotated,
    S: SourceLines<'a> + SourceMetadata<'a> + ?Sized,
    K: ErrorKind,
{
    /// Source whose coordinates interpret the annotation target.
    source: &'a S,

    /// Message and target reused as the report heading and primary location.
    annotation: A,

    /// Caller-selected classification for wording and styling metadata.
    kind: K,
}

impl<'a, A, K, S> OneshotKinded<'a, A, K, S>
where
    A: Annotated,
    S: SourceLines<'a> + SourceMetadata<'a> + ?Sized,
    K: ErrorKind,
{
    /// Reuse the annotation message as the heading under the supplied classification.
    #[inline]
    pub const fn from_raw_parts(source: &'a S, annotation: A, kind: K) -> Self {
        Self { source, annotation, kind }
    }

    /// Caller-selected descriptor and semantic metadata for this diagnostic.
    #[inline]
    pub const fn kind(&self) -> &K {
        let &Self { ref kind, .. } = self;

        kind
    }

    /// Source against which the primary annotation target is interpreted.
    #[inline]
    pub const fn source(&self) -> &'a S {
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
    fn list(&self) -> Option<RefOneOrMore<'_, Self::Annotation>> {
        let Self { annotation, .. } = self;

        Some(RefOneOrMore::single(annotation))
    }
}
