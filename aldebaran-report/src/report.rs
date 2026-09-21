//! This module contains the various error compoenents that are used to build an
//! error report.
//!
//! The main component is the [`Report`] trait, which is used to represent an
//! error that can be reported.

pub mod dynamic;

pub mod oneshot;

pub mod kind;

pub mod connotate;

pub mod severity;

use alloc::boxed::Box;
use core::convert::Infallible;

use aldebaran_source::prelude::{SourceLines, SourceMetadata};
use aldebaran_span::span::Span;

use crate::{
    annotated::Annotations,
    attached::Attached,
    prelude::{Annotated, Label, Title},
};

/*
TODO: derive(Report):

we should be able to specify assoc params in the following style:

#[report(title = "An error occurred")]

or..

#[report(title)]
field: type

we can support this by specializing support for literal expressions in the title position, which allows us to use a constant value as the title, without requiring more complex syntax.

same goes for any kind of field, really.

however, #[report(annotation)] denotes the first field annotation, #[report(annotation[...])] the rest field.

#[report(annotation[...])] is optional, and will default to an empty slice if not present.

a top-level #[report(annotation = "Annotation message")] attribute can be used for static annotation messages, but this requires an additional #[report(annotation(span))] attribute to denote the field holding the span for the annotation.

same goes for #[report(source)] and #[report(kind)], albeit source can't be defaulted to a constant value.

TODO: redo this

we can have a &'static str as the title if the item itself has a `#[report(title = "this is the title")]` attribute on the top, otherwise, fall back to a regular field attr

friendly reminder that we have reports act both as a valid reportable entity and a regular error
*/

/// A source-independent diagnostic report.
///
/// Implementations expose diagnostic state already stored by the error value.
/// Querying a report does not materialize annotations or allocate storage.
pub trait Report {
    /// Diagnostic title type.
    type Title: Title + ?Sized;

    /// Diagnostic kind type.
    type Kind: kind::ErrorKind;

    /// Inline annotation storage exposed by this report.
    type Annotations: Annotations;

    /// Retrieve the diagnostic kind.
    fn kind(&self) -> &Self::Kind;

    /// Retrieve the diagnostic title.
    fn title(&self) -> &Self::Title;

    /// Borrow the diagnostic annotation storage.
    fn annotations(&self) -> &Self::Annotations;

    /// Retrieve the smallest source span covering every annotation.
    ///
    /// [`Option::None`] means the report exposes no annotations. Individual
    /// annotations always carry a concrete [`Span`].
    #[inline]
    fn target(&self) -> Option<Span> {
        self.annotations()
            .list()
            .into_iter()
            .flat_map(|annotations| annotations.iter())
            .map(Annotated::target)
            .reduce(|left, right| left.superset(right))
    }

    /// Borrow this report together with one concrete rendering source.
    ///
    /// Attachment stores only references and does not copy or allocate report state.
    #[inline]
    fn attach<'source, 'report, S>(&'report self, source: &'source S) -> Attached<'source, 'report, S, Self>
    where
        Self: Sized,
        S: SourceLines<'source> + SourceMetadata<'source> + ?Sized,
    {
        Attached::new(source, self)
    }
}

/// A report borrowed together with a concrete rendering source.
pub trait SourceReport<'source>: Report {
    /// Source used to render this report.
    type Source: SourceLines<'source> + SourceMetadata<'source> + ?Sized;

    /// Retrieve the rendering source.
    fn source(&self) -> &Self::Source;
}

/// Assert that the source-independent report trait is object-safe.
#[doc(hidden)]
fn _report_must_be_object_safe()
where
    dyn Report<Title = str, Kind = kind::Simple, Annotations = Label<&'static str>>: Report,
{
}

impl<E> Report for &E
where
    E: Report + ?Sized,
{
    type Title = E::Title;

    type Kind = E::Kind;

    type Annotations = E::Annotations;

    #[inline]
    fn kind(&self) -> &Self::Kind {
        E::kind(self)
    }

    #[inline]
    fn title(&self) -> &Self::Title {
        E::title(self)
    }

    #[inline]
    fn annotations(&self) -> &Self::Annotations {
        E::annotations(self)
    }
}

impl<E> Report for Box<E>
where
    E: Report + ?Sized,
{
    type Title = E::Title;

    type Kind = E::Kind;

    type Annotations = E::Annotations;

    #[inline]
    fn kind(&self) -> &Self::Kind {
        E::kind(self)
    }

    #[inline]
    fn title(&self) -> &Self::Title {
        E::title(self)
    }

    #[inline]
    fn annotations(&self) -> &Self::Annotations {
        E::annotations(self)
    }
}

impl<'source, E> SourceReport<'source> for &E
where
    E: SourceReport<'source> + ?Sized,
{
    type Source = E::Source;

    #[inline]
    fn source(&self) -> &Self::Source {
        E::source(self)
    }
}

impl Report for Infallible {
    type Title = Self;

    type Kind = kind::Simple;

    type Annotations = Label<&'static str>;

    #[inline]
    fn kind(&self) -> &Self::Kind {
        match *self {}
    }

    #[inline]
    fn title(&self) -> &Self::Title {
        match *self {}
    }

    #[inline]
    fn annotations(&self) -> &Self::Annotations {
        match *self {}
    }
}
