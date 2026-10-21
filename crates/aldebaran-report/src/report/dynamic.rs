//! Report forms for annotation collections assembled at runtime.
//!
//! [`DynamicError`] accommodates diagnostics whose annotation count is not known
//! in advance. When the number of related annotations is fixed instead,
//! [`InlineAnnotations`](crate::annotated::InlineAnnotations) provides inline
//! storage for that collection.

use core::{
    clone::Clone,
    cmp::{Eq, PartialEq},
    default::Default,
    hash,
};

use aldebaran_baseline::prelude::{Nonempty, RefNonempty};

use aldebaran_source::prelude::{Source, SourceLines, SourceMetadata};

use crate::{
    annotated::Annotations,
    prelude::{Annotated, ErrorKind, Report, Simple, Title},
    report::SourceReport,
};

/// An owned annotation collection paired with a borrowed source.
///
/// The primary annotation remains present while related annotations can grow or
/// shrink through [`Self::annotations_mut`], which may allocate when growing the tail.
#[derive(Debug)]
// NOTE(invariant): None.
pub struct DynamicError<'source, S, A, T, K = Simple>
where
    S: Source<'source>,
    A: Annotated,
    T: Title,
    K: ErrorKind,
{
    /// Source text associated with this report.
    source: &'source S,

    /// Nonempty annotation collection attached to the report.
    annotations: Nonempty<A>,

    /// Human-readable report title.
    title: T,

    /// Structured report kind.
    kind: K,
}

impl<'source, S, A, T, K> Clone for DynamicError<'source, S, A, T, K>
where
    S: Source<'source>,
    A: Annotated,
    T: Title,
    K: ErrorKind,
    Nonempty<A>: Clone,
    T: Clone,
    K: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let &Self {
            source,
            ref annotations,
            ref title,
            ref kind,
        } = self;

        let annotations = annotations.clone();
        let title = title.clone();
        let kind = kind.clone();

        Self {
            source,
            annotations,
            title,
            kind,
        }
    }
}

impl<'a, S, A, T, K> Eq for DynamicError<'a, S, A, T, K>
where
    S: Source<'a> + 'a,
    A: Annotated,
    T: Title,
    K: ErrorKind,
    S: Eq,
    Nonempty<A>: Eq,
    T: Eq,
    K: Eq,
{
}

impl<'a, S, A, T, K> PartialEq for DynamicError<'a, S, A, T, K>
where
    S: Source<'a> + 'a,
    A: Annotated,
    T: Title,
    K: ErrorKind,
    S: PartialEq,
    Nonempty<A>: PartialEq,
    T: PartialEq,
    K: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let &Self {
            source,
            ref annotations,
            ref title,
            ref kind,
        } = self;

        let &Self {
            source: other_source,
            annotations: ref other_annotations,
            title: ref other_title,
            kind: ref other_kind,
        } = other;

        source == other_source && annotations == other_annotations && title == other_title && kind == other_kind
    }
}

impl<'source, S, A, T, K> hash::Hash for DynamicError<'source, S, A, T, K>
where
    S: Source<'source>,
    A: Annotated,
    T: Title,
    K: ErrorKind,
    S: hash::Hash,
    Nonempty<A>: hash::Hash,
    T: hash::Hash,
    K: hash::Hash,
{
    #[inline]
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
        let &Self {
            source,
            ref annotations,
            ref title,
            ref kind,
        } = self;

        source.hash(state);
        annotations.hash(state);
        title.hash(state);
        kind.hash(state);
    }
}

impl<'a, S, A, T, K> DynamicError<'a, S, A, T, K>
where
    S: Source<'a> + 'a,
    A: Annotated,
    T: Title,
    K: ErrorKind,
{
    /// Source whose coordinates interpret the stored annotation targets.
    #[inline]
    pub const fn source(&self) -> &'a S {
        let &Self { source, .. } = self;

        source
    }

    /// Heading displayed independently of the annotation messages.
    #[inline]
    pub const fn title(&self) -> &T {
        let &Self { ref title, .. } = self;

        title
    }

    /// Classification supplying descriptor text and styling metadata.
    #[inline]
    pub const fn kind(&self) -> &K {
        let &Self { ref kind, .. } = self;

        kind
    }

    /// Primary annotation followed by related context in caller order.
    #[inline]
    pub const fn annotations(&self) -> &Nonempty<A> {
        let &Self { ref annotations, .. } = self;

        annotations
    }

    /// Edit the primary annotation or grow and shrink its related context.
    ///
    /// The collection retains a primary value even when its tail is cleared.
    /// Growing the tail may allocate.
    #[inline]
    pub const fn annotations_mut(&mut self) -> &mut Nonempty<A> {
        let &mut Self { ref mut annotations, .. } = self;

        annotations
    }
}

impl<'a, S, A, T, K> DynamicError<'a, S, A, T, K>
where
    S: Source<'a> + 'a,
    A: Annotated,
    T: Title,
    K: ErrorKind,
{
    /// Start with a primary annotation and the kind type's default classification.
    #[inline]
    pub fn new(source: &'a S, title: T, annotation: A) -> Self
    where
        K: Default,
    {
        let annotations = Nonempty::single(annotation);

        let kind: K = Default::default();

        Self {
            source,
            annotations,
            title,
            kind,
        }
    }

    /// Start with a primary annotation and an explicit diagnostic classification.
    #[inline]
    pub const fn new_with_kind(source: &'a S, title: T, annotation: A, kind: K) -> Self {
        let annotations = Nonempty::single(annotation);

        Self {
            source,
            annotations,
            title,
            kind,
        }
    }

    /// Retain an existing primary and related collection under the default kind.
    #[inline]
    pub fn new_with_annotations(source: &'a S, title: T, annotations: Nonempty<A>) -> Self
    where
        K: Default,
    {
        let kind: K = Default::default();

        Self {
            source,
            annotations,
            title,
            kind,
        }
    }

    /// Retain the supplied collection order and diagnostic classification.
    #[inline]
    pub const fn from_raw_parts(source: &'a S, title: T, annotations: Nonempty<A>, kind: K) -> Self {
        Self {
            source,
            annotations,
            title,
            kind,
        }
    }
}

impl<'a, S, A, T, K> Report for DynamicError<'a, S, A, T, K>
where
    S: SourceLines<'a> + SourceMetadata<'a>,
    A: Annotated,
    T: Title,
    K: ErrorKind,
{
    type Title = T;

    type Kind = K;

    type Annotations = Self;

    #[inline]
    fn title(&self) -> &Self::Title {
        let Self { title, .. } = self;

        title
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

impl<'a, S, A, T, K> SourceReport<'a> for DynamicError<'a, S, A, T, K>
where
    S: SourceLines<'a> + SourceMetadata<'a>,
    A: Annotated,
    T: Title,
    K: ErrorKind,
{
    type Source = S;

    #[inline]
    fn source(&self) -> &Self::Source {
        let &Self { source, .. } = self;

        source
    }
}

impl<'a, S, A, T, K> Annotations for DynamicError<'a, S, A, T, K>
where
    S: SourceLines<'a> + SourceMetadata<'a>,
    A: Annotated,
    T: Title,
    K: ErrorKind,
{
    type Annotation = A;

    #[inline]
    fn list(&self) -> Option<RefNonempty<'_, Self::Annotation>> {
        let Self { annotations, .. } = self;

        Some(annotations.as_ref())
    }
}
