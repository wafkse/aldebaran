//! Integration tests for the allocation-free report derive.

use aldebaran::report::codegen::Report;
use aldebaran::report::prelude::{Annotated, Annotations, InlineAnnotations, Label, Report as ReportTrait, SourceReport};
use aldebaran::span::prelude::Span;

#[derive(Debug, Report)]
#[error("unexpected token")]
#[report(title = "unexpected token", message = "expected an expression")]
struct Unexpected(Span);

#[derive(Debug, Report)]
#[error("malformed number")]
#[report(title = "malformed number", message = "not a valid digit")]
struct BadNumber(Span);

#[derive(Debug, Report)]
enum LexError {
    #[error(transparent(0))]
    #[report(transparent)]
    Unexpected(Unexpected),

    #[error(transparent(0))]
    #[report(transparent)]
    BadNumber(BadNumber),
}

#[test]
fn derives_error_and_report_for_struct() {
    let error = Unexpected(Span::MIN);

    assert_eq!(ReportTrait::title(&error), "unexpected token");
    assert_eq!(Annotated::message(&error), "expected an expression");
    assert_eq!(Annotated::target(&error), Span::MIN);
    assert_eq!(error.to_string(), "unexpected token");

    fn assert_error<E: core::error::Error>(_error: &E) {}

    assert_error(&error);
}

#[test]
fn enum_forwards_to_active_variant() {
    let errors = [
        LexError::Unexpected(Unexpected(Span::MIN)),
        LexError::BadNumber(BadNumber(Span::MIN)),
    ];

    assert_eq!(ReportTrait::title(&errors[0]), "unexpected token");
    assert_eq!(Annotated::message(&errors[0]), "expected an expression");
    assert_eq!(ReportTrait::title(&errors[1]), "malformed number");
    assert_eq!(Annotated::message(&errors[1]), "not a valid digit");
}

#[test]
fn source_attachment_only_borrows_report_and_source() {
    let error = Unexpected(Span::MIN);
    let attached = ReportTrait::attach(&error, "let x = ;");

    assert_eq!(SourceReport::source(&attached), "let x = ;");
    assert_eq!(ReportTrait::title(&attached), "unexpected token");
    assert_eq!(Annotated::target(&error), Span::MIN);
}

#[derive(Debug, Report)]
#[error("expected expression")]
#[report(title = "missing expression", message = "expected expression")]
struct Direct(Span);

#[derive(Debug, Report)]
#[error(transparent(0))]
#[report(transparent)]
struct Transparent(Direct);

#[test]
fn transparent_struct_preserves_report_contract() {
    let report = Transparent(Direct(Span::MIN));

    assert_eq!(ReportTrait::title(&report), "missing expression");
    assert_eq!(Annotated::message(&report), "expected expression");
    assert_eq!(Annotated::target(&report), Span::MIN);
    assert_eq!(report.to_string(), "expected expression");
}

#[derive(Debug, Report)]
#[error("wrapped diagnostic")]
#[report(transparent(inner))]
struct WithContext {
    context: u8,

    inner: Unexpected,
}

#[test]
fn transparent_struct_can_select_one_report_field() {
    let inner = Unexpected(Span::MIN);
    let report = WithContext { context: 7, inner };

    assert_eq!(report.context, 7);
    assert_eq!(ReportTrait::title(&report), "unexpected token");
    assert_eq!(Annotated::message(&report), "expected an expression");
    assert_eq!(Annotated::target(&report), Span::MIN);
}

#[derive(Debug, Report)]
#[error("stored diagnostic")]
#[report(title_with = Stored::report_title, annotations = annotations)]
struct Stored {
    title: &'static str,

    annotations: InlineAnnotations<Label<&'static str>, 1>,
}

impl Stored {
    const fn report_title(&self) -> &str {
        let Self { title, .. } = self;

        title
    }
}

#[test]
fn report_can_reuse_inline_annotation_storage() {
    let primary = Label::new("expected expression", Span::MIN);
    let related = Label::new("while parsing conditional", Span::MIN);
    let annotations = InlineAnnotations::new(primary, [related]);
    let report = Stored {
        title: "conditional statement",
        annotations,
    };
    let annotations = ReportTrait::annotations(&report).list();

    assert_eq!(ReportTrait::title(&report), "conditional statement");
    assert_eq!(*Annotated::message(&report), "expected expression");

    match annotations {
        Some(annotations) => {
            assert_eq!(*annotations.first().message(), "expected expression");
            assert_eq!(*annotations.rest()[0].message(), "while parsing conditional");
        }
        None => panic!("inline annotations always contain a primary annotation"),
    }
}
