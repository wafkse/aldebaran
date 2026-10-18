//! Compact report forms for diagnostics assembled at the call site.
//!
//! Their title can be supplied separately or taken from an annotation, and the
//! kind can use its default or a caller-selected value.

mod concise;
mod kinded;
mod many;

pub use concise::OneshotConcise;

pub use kinded::OneshotKinded;

use aldebaran_baseline::prelude::RefOneOrMore;
use aldebaran_source::{line::SourceLines, prelude::SourceMetadata};
pub use many::OneshotMany;

use crate::{
    annotated::Annotations,
    prelude::{Annotated, ErrorKind, Label, Report, Simple, Title},
    report::SourceReport,
};

/// A source-aware report with a separate title and one annotation.
///
/// It uses the default [`Simple::Error`] kind.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
// NOTE(invariant): None.
pub struct Oneshot<'source, T, A, S = str>
where
    T: Title,
    A: Annotated,
    S: SourceLines<'source> + SourceMetadata<'source> + ?Sized,
{
    /// Heading displayed independently of the annotation message.
    title: T,

    /// Source whose coordinates interpret the annotation target.
    source: &'source S,

    /// Message and target identifying the diagnostic location.
    annotation: A,
}

impl<'source, T, A, S> Oneshot<'source, T, A, S>
where
    T: Title,
    A: Annotated,
    S: SourceLines<'source> + SourceMetadata<'source> + ?Sized,
{
    /// Use a separate heading and primary annotation under the error classification.
    #[inline]
    #[doc(alias = "from_raw_parts")]
    pub const fn new(title: T, source: &'source S, annotation: A) -> Self {
        Self { title, source, annotation }
    }

    /// Heading displayed independently of the primary annotation message.
    #[inline]
    pub const fn title(&self) -> &T {
        let &Self { ref title, .. } = self;

        title
    }

    /// Source against which the primary annotation target is interpreted.
    #[inline]
    pub const fn source(&self) -> &'source S {
        let &Self { source, .. } = self;

        source
    }

    /// Location and message displayed beneath the separate report heading.
    #[inline]
    pub const fn annotation(&self) -> &A {
        let &Self { ref annotation, .. } = self;

        annotation
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

impl<'source, T, A, S> SourceReport<'source> for Oneshot<'source, T, A, S>
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

/// Use the annotation message as the report title and its span as the target.
#[inline]
pub const fn single<'source, A, S>(source: &'source S, annotation: A) -> OneshotConcise<'source, A, S>
where
    A: Annotated,
    S: SourceLines<'source> + SourceMetadata<'source> + ?Sized,
{
    OneshotConcise::from_raw_parts(source, annotation)
}

/// Build a concise report whose label covers the source footprint.
///
/// For an empty source, the label uses [`crate::reexport::Span::MIN`] as the
/// boundary at zero.
#[inline]
pub fn all<'source, T, S>(source: &'source S, reason: T) -> OneshotConcise<'source, Label<T>, S>
where
    T: Title,
    S: SourceLines<'source> + SourceMetadata<'source> + ?Sized,
{
    let annotation = Label::all(source, reason);

    OneshotConcise::from_raw_parts(source, annotation)
}

/// Build a report whose title comes from the annotation and whose kind is
/// supplied by the caller.
#[inline]
pub const fn kinded<'source, A, K, S>(source: &'source S, annotation: A, kind: K) -> OneshotKinded<'source, A, K, S>
where
    A: Annotated,
    K: ErrorKind,
    S: SourceLines<'source> + SourceMetadata<'source> + ?Sized,
{
    OneshotKinded::from_raw_parts(source, annotation, kind)
}

/// Build a report with a separate title and a nonempty annotation collection.
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
