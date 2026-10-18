use core::ptr;

use alloc::boxed::Box;

use aldebaran_baseline::{nonempty::RefOneOrMore, span::Span};

use crate::annotated::{Annotations, InlineAnnotations, label::Label};

use super::{Report, SourceReport, kind::Simple};

/// A source-independent diagnostic with an optional inline annotation set.
// NOTE(invariant): None.
struct Diagnostic(Option<InlineAnnotations<Label<&'static str>, 2>>);

impl Annotations for Diagnostic {
    type Annotation = Label<&'static str>;

    #[inline]
    fn list(&self) -> Option<RefOneOrMore<'_, Self::Annotation>> {
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
