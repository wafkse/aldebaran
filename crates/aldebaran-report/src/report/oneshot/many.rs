//! One-shot reports over a nonempty annotation collection.
//!
//! Use [`OneshotMany`] when a diagnostic needs a separate title and multiple
//! source annotations.

use aldebaran_baseline::prelude::RefNonempty;

use aldebaran_source::prelude::{SourceLines, SourceMetadata};

use crate::{
    annotated::Annotations,
    prelude::{Annotated, Report, Simple, Title},
    report::SourceReport,
};

/// A one-shot diagnostic over a borrowed, nonempty annotation collection.
///
/// The title is separate from the annotations, and the kind is
/// [`Simple::Error`].
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
// NOTE(invariant): None.
pub struct OneshotMany<'source, 'borrow, S, T, A>
where
    A: Annotated,
    T: Title,
    S: SourceLines<'source> + SourceMetadata<'source> + ?Sized,
    'source: 'borrow,
{
    /// Source whose coordinates interpret all annotation targets.
    source: &'source S,

    /// Heading displayed independently of annotation messages.
    title: T,

    /// Borrowed primary annotation followed by caller-ordered related context.
    annotation: RefNonempty<'borrow, A>,
}

impl<'source, 'borrow, S, T, A> OneshotMany<'source, 'borrow, S, T, A>
where
    A: Annotated,
    T: Title,
    S: SourceLines<'source> + SourceMetadata<'source> + ?Sized,
    'source: 'borrow,
{
    /// Retain the borrowed annotation order under a separate heading and error kind.
    #[inline]
    pub const fn raw(source: &'source S, title: T, annotation: RefNonempty<'borrow, A>) -> Self {
        Self { source, title, annotation }
    }

    /// Source against which the primary and related targets are interpreted.
    #[inline]
    pub const fn source(&self) -> &'source S {
        let &Self { source, .. } = self;

        source
    }

    /// Caller-ordered annotations with their original input lifetime.
    #[inline]
    pub const fn annotations(&self) -> RefNonempty<'borrow, A> {
        let &Self { annotation, .. } = self;

        annotation
    }
}

impl<'source, 'borrow, S, T, A> Report for OneshotMany<'source, 'borrow, S, T, A>
where
    A: Annotated,
    T: Title,
    S: SourceLines<'source> + SourceMetadata<'source> + ?Sized,
    'source: 'borrow,
{
    type Title = T;

    type Kind = Simple;

    type Annotations = Self;

    #[inline]
    fn title(&self) -> &Self::Title {
        let Self { title, .. } = self;

        title
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

impl<'source, 'borrow, S, T, A> SourceReport<'source> for OneshotMany<'source, 'borrow, S, T, A>
where
    A: Annotated,
    T: Title,
    S: SourceLines<'source> + SourceMetadata<'source> + ?Sized,
    'source: 'borrow,
{
    type Source = S;

    #[inline]
    fn source(&self) -> &Self::Source {
        let &Self { source, .. } = self;

        source
    }
}

impl<'source, 'borrow, S, T, A> Annotations for OneshotMany<'source, 'borrow, S, T, A>
where
    A: Annotated,
    T: Title,
    S: SourceLines<'source> + SourceMetadata<'source> + ?Sized,
    'source: 'borrow,
{
    type Annotation = A;

    #[inline]
    fn list(&self) -> Option<RefNonempty<'_, Self::Annotation>> {
        let &Self { annotation, .. } = self;

        Some(annotation)
    }
}
