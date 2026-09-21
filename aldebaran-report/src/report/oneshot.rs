//! A module containing the [`Oneshot`] error type.
//!
//! The main rationale behind this error type is to provide a simple, one-shot
//! error type that can be in contexts where declaring a new error type can be
//! cumbersome.

mod concise;
mod kinded;
mod many;

pub use concise::OneshotConcise;

pub use kinded::OneshotKinded;

use aldebaran_dsa::prelude::RefOneOrMore;
use aldebaran_source::{line::SourceLines, prelude::SourceMetadata};
pub use many::OneshotMany;

use crate::{
    annotated::Annotations,
    prelude::{Annotated, ErrorKind, Label, Report, Simple, Title},
};

/// A simple, one-shot error type.
///
/// This contains a static, known amount of annotations, and is meant to be used
/// in contexts where a full-blown error type is not necessary.
///
/// Albeit many extraneous items refer to this type, there are other types that
/// provide the same functionality, but with other quirks regarding what data
/// they require.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Oneshot<'source, T, A, S = str>
where
    T: Title,
    A: Annotated,
    S: SourceLines<'source> + SourceMetadata<'source> + ?Sized,
{
    /// The title of this error.
    title: T,
    /// The source of this error.
    source: &'source S,
    /// The singular annotation of this error.
    annotation: A,
}

impl<'source, T, A, S> Oneshot<'source, T, A, S>
where
    T: Title,
    A: Annotated,
    S: SourceLines<'source> + SourceMetadata<'source> + ?Sized,
{
    /// Create a new [`Oneshot`] error with the given title, source, and
    /// annotation.
    #[inline]
    #[doc(alias = "from_raw_parts")]
    pub const fn new(title: T, source: &'source S, annotation: A) -> Self {
        Self { title, source, annotation }
    }

    /// Retrieve the title of this error.
    #[inline]
    pub const fn title(&self) -> &T {
        let &Self { ref title, .. } = self;

        title
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

impl<'source, T, A, S> Report for Oneshot<'source, T, A, S>
where
    T: Title,
    A: Annotated,
    S: SourceLines<'source> + SourceMetadata<'source> + ?Sized,
{
    type Title = T;

    type Kind = Simple;

    type Annotations = Self;

    #[inline]
    fn title(&self) -> &Self::Title {
        let &Self { ref title, .. } = self;

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

impl<'source, T, A, S> crate::report::SourceReport<'source> for Oneshot<'source, T, A, S>
where
    T: Title,
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

impl<'source, T, A, S> Annotations for Oneshot<'source, T, A, S>
where
    T: Title,
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

/// Create a new concise [`oneshot error`](Oneshot) where the title is
/// inferred from the annotation.
///
/// Note that this is not an [`Oneshot`] type but a [`OneshotConcise`]
/// type.
#[inline]
pub const fn single<'source, A, S>(source: &'source S, annotation: A) -> OneshotConcise<'source, A, S>
where
    A: Annotated,
    S: SourceLines<'source> + SourceMetadata<'source> + ?Sized,
{
    OneshotConcise::from_raw_parts(source, annotation)
}

/// Create a new [`oneshot error`](Oneshot) where the title is inferred from
/// the annotation and the error spans all of the available source.
///
/// The underlying annotation is a [`Label`] with the printable `P`.
#[inline]
pub fn all<'source, T, S>(source: &'source S, reason: T) -> OneshotConcise<'source, Label<T>, S>
where
    T: Title,
    S: SourceLines<'source> + SourceMetadata<'source> + ?Sized,
{
    let annotation = Label::all(source, reason);

    OneshotConcise::from_raw_parts(source, annotation)
}

/// Create a new kinded [`oneshot error`](Oneshot) where the title is
/// inferred from the annotation.
///
/// Note that this is not an [`Oneshot`] type but a [`OneshotKinded`]
/// type.
#[inline]
pub const fn kinded<'source, A, K, S>(source: &'source S, annotation: A, kind: K) -> OneshotKinded<'source, A, K, S>
where
    A: Annotated,
    K: ErrorKind,
    S: SourceLines<'source> + SourceMetadata<'source> + ?Sized,
{
    OneshotKinded::from_raw_parts(source, annotation, kind)
}

/// Create a new [`oneshot error`](Oneshot) with a title and an unspecified
/// number of annotations.
#[inline]
pub const fn many<'source, 'borrow, S, T, A>(
    source: &'source S,
    title: T,
    annotations: RefOneOrMore<'borrow, A>,
) -> OneshotMany<'source, 'borrow, S, T, A>
where
    S: SourceLines<'source> + SourceMetadata<'source> + ?Sized,
    T: Title,
    A: Annotated,
    'source: 'borrow,
{
    OneshotMany::from_raw_parts(source, title, annotations)
}
