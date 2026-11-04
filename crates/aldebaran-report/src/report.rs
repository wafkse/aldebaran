//! Diagnostic content that can be associated with source data for rendering.
//!
//! See [`Report`], [`SourceReport`], [`oneshot`], and [`dynamic::DynamicError`] for
//! further information.

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
    /// Diagnostic title type.
    type Title: Title + ?Sized;

    /// Diagnostic kind type.
    type Kind: kind::ErrorKind;

    /// Storage or adapter exposing the diagnostic's primary and related locations.
    type Annotations: Annotations;

    /// Retrieve the diagnostic kind.
    fn kind(&self) -> &Self::Kind;

    /// Retrieve the diagnostic title.
    fn title(&self) -> &Self::Title;

    /// Borrow the diagnostic annotation storage.
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
    /// The source's coordinates must match the annotation targets; source identity
    /// and target bounds are not checked.
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
/// The source's coordinates must match every target; source identity and target
/// bounds are not checked.
pub trait SourceReport<'source>: Report {
    /// Source used to render this report.
    type Source: SourceLines<'source> + SourceMetadata<'source> + ?Sized;

    /// Retrieve the rendering source.
    fn source(&self) -> &Self::Source;
}

/// Object-safety witness for [`Report`].
#[doc(hidden)]
const fn _report_must_be_object_safe()
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
mod tests {
    //! Tests module for the [`report`] module.
    //!
    //! [`report`]: super

    use core::ptr;

    use alloc::boxed::Box;
    use alloc::vec::Vec;

    use aldebaran_baseline::{nonempty::RefNonempty, span::Span};

    use crate::annotated::{Annotations, InlineAnnotations, label::Label};

    use super::{Report, SourceReport, dynamic::DynamicError, kind::Simple};

    /// A source-independent diagnostic with an optional inline annotation set.
    // NOTE(invariant): None.
    struct Diagnostic(Option<InlineAnnotations<Label<&'static str>, 2>>);

    impl Annotations for Diagnostic {
        type Annotation = Label<&'static str>;

        #[inline]
        fn list(&self) -> Option<RefNonempty<'_, Self::Annotation>> {
            let Self(annotations) = self;

            annotations.as_ref().and_then(Annotations::list)
        }
    }

    impl Report for Diagnostic {
        type Title = str;

        type Kind = Simple;

        type Annotations = Self;

        #[inline]
        fn title(&self) -> &Self::Title {
            "diagnostic"
        }

        #[inline]
        fn kind(&self) -> &Self::Kind {
            &Simple::Warning
        }

        #[inline]
        fn annotations(&self) -> &Self::Annotations {
            self
        }
    }

    #[test]
    fn unannotated_report_has_no_target() {
        let report = Diagnostic(None);

        assert_eq!(report.target(), None);
    }

    #[test]
    fn dynamic_annotations_grow_and_clear_without_losing_the_primary() {
        let source = "abcdef";
        let primary_span = Span::unit(1);
        let related_span = Span::unit(4);

        let primary = Label::new("primary", primary_span);
        let related = Label::new("related", related_span);
        let mut report = DynamicError::<_, _, _, Simple>::new(&source, "heading", primary);

        report.annotations_mut().list_mut().push(related);

        let annotations = report.annotations().as_ref();

        assert_eq!(annotations.iter().collect::<Vec<_>>(), [&primary, &related]);

        report.annotations_mut().list_mut().clear();

        assert_eq!(report.annotations().first(), &primary);
        assert_eq!(report.annotations().len().get(), 1);
    }

    #[test]
    fn target_covers_all_annotations_regardless_of_primary_position() {
        let first = Label::new("first", Span::unit(1));
        let middle = Label::new("middle", Span::unit(4));
        let last = Label::new("last", Span::unit(8));

        for [primary, left, right] in [[first, middle, last], [middle, last, first], [last, first, middle]] {
            let annotations = InlineAnnotations::new(primary, [left, right]);
            let report = Diagnostic(Some(annotations));

            assert_eq!(report.target().map(|span| span.range()), Some(1..9));
        }
    }

    #[test]
    fn references_boxes_and_trait_objects_preserve_report_semantics() {
        let first = Label::new("first", Span::unit(2));
        let last = Label::new("last", Span::unit(5));
        let annotations = InlineAnnotations::new(first, [last, first]);
        let report = Diagnostic(Some(annotations));
        let borrowed = &report;
        let boxed = Box::new(borrowed);
        let erased: &dyn Report<Title = str, Kind = Simple, Annotations = Diagnostic> = &boxed;

        assert_eq!(erased.title(), "diagnostic");
        assert_eq!(erased.kind(), &Simple::Warning);
        assert_eq!(erased.target().map(|span| span.range()), Some(2..6));
        assert!(
            ptr::eq(erased.annotations(), &report),
            "wrapping a report must retain its annotation storage"
        );
    }

    #[test]
    fn attaching_source_borrows_the_original_report_and_annotations() {
        let source = "abcdef";
        let first = Label::new("first", Span::unit(1));
        let last = Label::new("last", Span::unit(4));
        let annotations = InlineAnnotations::new(first, [last, first]);
        let report = Diagnostic(Some(annotations));
        let attached = report.attach(source);

        assert!(ptr::eq(attached.report(), &report), "attachment must borrow the original report");
        assert!(
            ptr::eq(SourceReport::source(&attached), source),
            "attachment must retain the source borrow"
        );
        assert!(
            ptr::eq(attached.annotations(), report.annotations()),
            "attachment must share annotation storage"
        );
        assert_eq!(attached.title(), report.title());
        assert_eq!(attached.kind(), report.kind());
        assert_eq!(attached.target().map(|span| span.range()), Some(1..5));
    }
}
