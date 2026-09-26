//! One-shot reports over a borrowed nonempty annotation collection.
//!
//! [`OneshotMany`] attaches a title and source to a collection of annotations
//! selected by the caller. The report borrows all inputs and exposes them through
//! ordinary reporting traits without copying annotation storage.

use aldebaran_dsa::prelude::RefOneOrMore;

use aldebaran_source::prelude::{SourceLines, SourceMetadata};

use crate::{
    annotated::Annotations,
    prelude::{Annotated, Report, Simple, Title},
};

/// A simple, concise one-shot error type.
///
/// This contains a static, known amount of annotations, and is meant to be used
/// in contexts where a full-blown error type is not necessary.
///
/// This is an hassle-free error type that is meant to be used in contexts where
/// a title is known, and one or more annotations are present.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
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

    /// The annotations of this error.
    annotation: RefOneOrMore<'borrow, A>,
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
    pub const fn from_raw_parts(source: &'source S, title: T, annotation: RefOneOrMore<'borrow, A>) -> Self {
        Self { source, title, annotation }
    }

    /// Retrieve the source of this error.
    #[inline]
    pub const fn source(&self) -> &'source S {
        let &Self { source, .. } = self;

        source
    }

    /// Retrieve the [`RefOneOrMore`] corresponding to the annotations of this
    /// error.
    #[inline]
    pub const fn annotations(&self) -> RefOneOrMore<'borrow, A> {
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

impl<'source, 'borrow, S, T, A> crate::report::SourceReport<'source> for OneshotMany<'source, 'borrow, S, T, A>
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
    fn list(&self) -> Option<RefOneOrMore<'_, Self::Annotation>> {
        let &Self { annotation, .. } = self;

        Some(annotation)
    }
}
