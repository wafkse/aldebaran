//! Diagnostic content that can be associated with source data for rendering.
//!
//! [`Report`] keeps headings, classification, and annotations independent of
//! source ownership. [`SourceReport`] adds the source needed to interpret those
//! locations. Use [`oneshot`] for diagnostics assembled at the call site or
//! [`dynamic::DynamicError`] when the annotation collection needs to grow.

pub mod dynamic;

pub mod oneshot;

pub mod kind;

pub mod connotate;

pub mod severity;

use alloc::boxed::Box;
use core::convert::Infallible;

use aldebaran_baseline::span::Span;
use aldebaran_source::prelude::{SourceLines, SourceMetadata};

use crate::{
    annotated::Annotations,
    attached::Attached,
    prelude::{Annotated, Label, Title},
};

/// Source-independent content describing a diagnostic and its locations.
///
/// The heading is separate from annotation messages, and classification supplies
/// printable wording and semantic metadata.
pub trait Report {
    /// Formatting contract for the diagnostic heading.
    type Title: Title + ?Sized;

    /// Classification supplying the heading descriptor and semantic metadata.
    type Kind: kind::ErrorKind;

    /// Storage or adapter exposing the diagnostic's primary and related locations.
    type Annotations: Annotations;

    /// Classification used for the heading descriptor and metadata-based styling.
    fn kind(&self) -> &Self::Kind;

    /// Heading text, independently of individual annotation messages.
    fn title(&self) -> &Self::Title;

    /// Locations and messages that relate this diagnostic to source content.
    fn annotations(&self) -> &Self::Annotations;

    /// Return the smallest source span covering every annotation.
    ///
    /// Returns `None` when there are no annotations.
    ///
    /// # Panics
    ///
    /// Panics if the bounding span cannot be represented.
    #[inline]
    fn target(&self) -> Option<Span> {
        self.annotations()
            .list()
            .into_iter()
            .flat_map(|annotations| annotations.iter())
            .map(Annotated::target)
            .reduce(|left, right| left.superset(right))
    }

    /// Interpret this report's targets against `source` without checking their bounds.
    ///
    /// The caller must choose a source whose coordinates match the annotation targets.
    #[inline]
    fn attach<'source, 'report, S>(&'report self, source: &'source S) -> Attached<'source, 'report, S, Self>
    where
        Self: Sized,
        S: SourceLines<'source> + SourceMetadata<'source> + ?Sized,
    {
        Attached::new(source, self)
    }
}

/// A diagnostic with the source needed to interpret its annotation targets.
///
/// Implementers must supply coordinates compatible with every target because
/// the trait does not check source identity or target bounds.
pub trait SourceReport<'source>: Report {
    /// Source whose coordinate system is used by every annotation target.
    type Source: SourceLines<'source> + SourceMetadata<'source> + ?Sized;

    /// Rendering input against which the report's targets are interpreted.
    fn source(&self) -> &Self::Source;
}

/// Object-safety witness for [`Report`].
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

#[cfg(test)]
mod tests;
