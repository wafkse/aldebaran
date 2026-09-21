//! Dynamic [`Report`] type.
//!
//! This module provides the [`DynamicError`] type, which is a dynamic error
//! type that allows an arbitrary number of annotations to be attached to it.
//!
//! This is useful for errors that are generated at runtime, where the number of
//! eventual annotations is not known at compile-time.
//!
//! The typology of the [`Annotation`] is not known either, and is given as a
//! generic parameter instead.
//!
//! [`Annotation`]: crate::prelude::Annotated

use core::{
    clone::Clone,
    cmp::{Eq, PartialEq},
    default::Default,
    hash,
};

use aldebaran_dsa::prelude::{OneOrMore, RefOneOrMore};

use aldebaran_source::prelude::{Source, SourceLines, SourceMetadata};

use crate::{
    annotated::Annotations,
    prelude::{Annotated, ErrorKind, Report, Simple, Title},
};

/// A dynamic [`Report`] type.
///
/// This type is a type that allows for an arbitrary type of [`Report`]
/// components to be attached to it.
#[derive(Debug)]
pub struct DynamicError<'source, S, A, T, K = Simple>
where
    S: Source<'source>,
    A: Annotated,
    T: Title,
    K: ErrorKind,
{
    source: &'source S,
    annotations: OneOrMore<A>,
    title: T,
    kind: K,
}

impl<'source, S, A, T, K> Clone for DynamicError<'source, S, A, T, K>
where
    S: Source<'source>,
    A: Annotated,
    T: Title,
    K: ErrorKind,
    OneOrMore<A>: Clone,
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
    OneOrMore<A>: Eq,
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
    OneOrMore<A>: PartialEq,
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
    OneOrMore<A>: hash::Hash,
    T: hash::Hash,
    K: hash::Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
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
    /// Retrieve the [`Source`] `S` attached to this error.
    #[inline]
    pub const fn source(&self) -> &'a S {
        let &Self { source, .. } = self;

        source
    }

    /// Retrieve the [`Title`] `T` attached to this error.
    #[inline]
    pub const fn title(&self) -> &T {
        let &Self { ref title, .. } = self;

        title
    }

    /// Retrieve the error [`Kind`] `K` attached to this error.
    ///
    /// [`Kind`]: ErrorKind
    #[inline]
    pub const fn kind(&self) -> &K {
        let &Self { ref kind, .. } = self;

        kind
    }

    /// Retrieve the underlying annotations attached to this error.
    #[inline]
    pub const fn annotations(&self) -> &OneOrMore<A> {
        let &Self { ref annotations, .. } = self;

        annotations
    }

    /// Retrieve the underlying annotations attached to this error in a mutable
    /// manner.
    #[inline]
    pub const fn annotations_mut(&mut self) -> &mut OneOrMore<A> {
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
    /// Create a new [`DynamicError`] with the given source, title, and
    /// primary annotation.
    ///
    /// Error kind is left to the default value for its type.
    #[inline]
    pub fn new(source: &'a S, title: T, annotation: A) -> Self
    where
        K: Default,
    {
        let annotations = OneOrMore::single(annotation);

        let kind: K = Default::default();

        Self {
            source,
            annotations,
            title,
            kind,
        }
    }

    /// Create a new [`DynamicError`] with the given source, title, primary
    /// annotation, and its error kind.
    #[inline]
    pub fn new_with_kind(source: &'a S, title: T, annotation: A, kind: K) -> Self {
        let annotations = OneOrMore::single(annotation);

        Self {
            source,
            annotations,
            title,
            kind,
        }
    }

    /// Create a new [`DynamicError`] with the given source, title, annotations.
    ///
    /// Error kind is left to the default value for its type.
    #[inline]
    pub fn new_with_annotations(source: &'a S, title: T, annotations: OneOrMore<A>) -> Self
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

    /// Create a new [`DynamicError`] from its individual components: source,
    /// title, annotations, and kind.
    #[inline]
    pub const fn from_raw_parts(source: &'a S, title: T, annotations: OneOrMore<A>, kind: K) -> Self {
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

impl<'a, S, A, T, K> crate::report::SourceReport<'a> for DynamicError<'a, S, A, T, K>
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
    fn list(&self) -> Option<RefOneOrMore<'_, Self::Annotation>> {
        let Self { annotations, .. } = self;

        Some(annotations.as_ref())
    }
}
