//! One-shot reports over a nonempty annotation collection.
//!
//! See [`OneshotMany`] for further information.

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
    /// The source of this error.
    source: &'source S,

    /// The title for this error.
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
    /// Instantiate a new [`OneshotMany`] error from its raw components.
    #[inline]
    pub const fn raw(source: &'source S, title: T, annotation: RefNonempty<'borrow, A>) -> Self {
        Self { source, title, annotation }
    }

    /// Retrieve the source of this error.
    #[inline]
    pub const fn source(&self) -> &'source S {
        let &Self { source, .. } = self;

        source
    }

    /// Retrieve the [`RefNonempty`] corresponding to the annotations of this error.
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
