//! Typed messages and independent error derives for report implementations.

use core::fmt;

use aldebaran_baseline::span::Span;
use aldebaran_print::prelude::Print;
use fack::prelude::Error;

use crate::{
    annotated::{Annotated, label::Label, title::Title},
    codegen::Report,
    render::{
        RenderMut,
        textual::{
            Present, Textual,
            offload::{
                fancy::{Fancy, FancySettings},
                summarized::Summarized,
            },
        },
    },
    report::Report as ReportTrait,
};

/// A report whose primary annotation retains its printable message type.
#[derive(Report)]
#[aldebaran(crate)]
#[report(title = "typed heading", annotations = label)]
struct TypedIssue<T>
where
    T: Title,
{
    /// Stored primary annotation.
    label: Label<T>,
}

/// Variant messages that require different printing contexts.
#[derive(Report)]
#[aldebaran(crate)]
enum ContextualReport {
    #[report(transparent)]
    Names(TypedIssue<Name>),

    #[report(transparent)]
    Numbers(TypedIssue<u8>),
}

/// Typed annotation messages alongside custom diagnostic wording.
#[derive(Report)]
#[aldebaran(crate)]
enum MixedReport {
    #[report(transparent)]
    Number(TypedIssue<u8>),

    #[report(transparent)]
    Text(TypedIssue<&'static str>),

    #[report(title = "custom heading", message = "custom message")]
    Custom(Span),
}

/// External printing data deliberately lacking a default value.
struct Names(&'static str);

/// A message requiring explicit external printing data.
struct Name;

impl Print for Name {
    type Context = Names;

    fn print_with_context<W>(&self, writer: &mut W, context: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        let Names(name) = context;

        writer.write_str(name)
    }
}

/// Source error whose behavior is independent of diagnostic generation.
#[derive(Debug, Error)]
#[error("root cause")]
struct Cause;

/// Standard error behavior supplied by a separate derive.
#[derive(Debug, Error, Report)]
#[aldebaran(crate)]
#[error("standard error display")]
#[error(source(cause))]
#[report(title = "diagnostic heading", message = "diagnostic annotation")]
struct IndependentError {
    /// Ordinary error source preserved by the error derive.
    cause: Cause,

    /// Source location used by the report derive.
    span: Span,
}

/// Reversing derive order preserves independent transparent behavior.
#[derive(Debug, Report, Error)]
#[aldebaran(crate)]
#[report(transparent)]
#[error(transparent)]
struct TransparentError(IndependentError);

/// A bound implemented by the diagnostic rather than its borrowed printer.
trait Marker {}

/// Bounds and message paths retain the meaning of the input enum's `Self`.
#[derive(Report)]
#[aldebaran(crate)]
enum SelfBound
where
    Self: Marker,
{
    #[report(transparent)]
    Wrapped(TypedIssue<u8>),

    #[report(title = "projected heading", message_with = Self::projected)]
    Projected(Span),
}

impl Marker for SelfBound {}

impl SelfBound {
    /// Project the annotation text from the custom variant's payload.
    fn projected(_: &Span) -> &str {
        "enum projection"
    }
}

#[test]
fn enum_self_bounds_and_projections_keep_their_original_subject() {
    let wrapped = SelfBound::Wrapped(TypedIssue {
        label: Label::new(42_u8, Span::unit(1)),
    });
    let projected = SelfBound::Projected(Span::unit(2));

    assert_eq!(Annotated::message(&wrapped).content().display().to_string(), "42");
    assert_eq!(Annotated::message(&projected).content().display().to_string(), "enum projection");
}

#[test]
fn standard_error_implementations_are_independent_of_reporting() {
    let diagnostic = IndependentError {
        cause: Cause,
        span: Span::unit(2),
    };
    let error: &dyn Error = &diagnostic;

    assert_eq!(error.to_string(), "standard error display");
    assert_eq!(error.source().map(ToString::to_string).as_deref(), Some("root cause"));
    assert_eq!(ReportTrait::title(&diagnostic), "diagnostic heading");
    assert_eq!(Annotated::message(&diagnostic), "diagnostic annotation");

    let transparent = TransparentError(diagnostic);
    let error: &dyn Error = &transparent;

    assert_eq!(error.to_string(), "standard error display");
    assert_eq!(error.source().map(ToString::to_string).as_deref(), Some("root cause"));
    assert_eq!(ReportTrait::title(&transparent), "diagnostic heading");
    assert_eq!(Annotated::message(&transparent), "diagnostic annotation");
}

#[test]
fn variant_messages_receive_their_printing_contexts() {
    let report = ContextualReport::Names(TypedIssue {
        label: Label::new(Name, Span::unit(1)),
    });
    let context = (Names("resolved name"), ((), ()));
    let mut output = String::new();

    Annotated::message(&report)
        .content()
        .print_with_context(&mut output, &context)
        .expect("writing to a String succeeds");

    assert_eq!(output, "resolved name");

    let report = ContextualReport::Numbers(TypedIssue {
        label: Label::new(42_u8, Span::unit(1)),
    });

    output.clear();
    Annotated::message(&report)
        .content()
        .print_with_context(&mut output, &context)
        .expect("writing to a String succeeds");

    assert_eq!(output, "42");
}

#[test]
fn heterogeneous_messages_preserve_content_headings_and_targets() {
    let number = MixedReport::Number(TypedIssue {
        label: Label::new(42_u8, Span::unit(1)),
    });
    let text = MixedReport::Text(TypedIssue {
        label: Label::new("text detail", Span::unit(3)),
    });
    let custom = MixedReport::Custom(Span::unit(5));

    for (report, message, heading, target) in [
        (number, "42", "typed heading", Span::unit(1)),
        (text, "text detail", "typed heading", Span::unit(3)),
        (custom, "custom message", "custom heading", Span::unit(5)),
    ] {
        assert_eq!(Annotated::message(&report).content().display().to_string(), message);
        assert_eq!(ReportTrait::title(&report), heading);
        assert_eq!(Annotated::target(&report), target);
        assert_eq!(ReportTrait::target(&report), Some(target));
    }
}

#[test]
fn renderers_include_the_forwarded_typed_message() {
    let report = MixedReport::Number(TypedIssue {
        label: Label::new(42_u8, Span::unit(0)),
    });
    let attached = report.attach("x");
    let mut summary = String::new();
    let mut fancy = String::new();

    Textual::<'_, _, Summarized, _>::new(&mut summary)
        .render_mut_with_input(Present::with_input(()), &attached)
        .expect("render summarized typed message");
    Textual::<'_, _, Fancy, _>::new(&mut fancy)
        .render_mut_with_input(Present::with_input(FancySettings::standard()), &attached)
        .expect("render fancy typed message");

    assert!(summary.contains("typed heading"), "summary lost the heading: {summary}");
    assert!(summary.contains(":: 42"), "summary lost the typed message: {summary}");
    assert!(fancy.contains("typed heading"), "fancy output lost the heading: {fancy}");
    assert!(fancy.contains(": 42"), "fancy output lost the typed message: {fancy}");
}
