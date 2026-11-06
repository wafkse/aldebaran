//! Behavior tests for generated report implementations.

#![cfg(feature = "codegen")]

#[cfg(test)]
mod tests {
    use fack::prelude::Error;

    use aldebaran_report::{
        annotated::{Annotated, InlineAnnotations, label::Label},
        codegen::Report,
        reexport::Span,
        report::Report as ReportTrait,
    };

    #[derive(Debug, Error, Report)]
    #[aldebaran(crate)]
    #[error("invalid value {value}")]
    #[report(title = "invalid value", message_with = issue_message)]
    struct GenericIssue<T> {
        target: Span,
        value: T,
    }

    fn issue_message<T>(_: &GenericIssue<T>) -> &'static str {
        "expected a valid value"
    }

    #[derive(Debug, Error, Report)]
    #[aldebaran(crate)]
    #[error("multiple labels")]
    #[report(title = "multiple labels", annotations = labels)]
    struct MultiIssue {
        labels: InlineAnnotations<Label<&'static str>, 1>,
    }

    #[derive(Debug, Error, Report)]
    #[aldebaran(crate)]
    #[error(transparent(inner))]
    #[report(transparent(inner))]
    struct WrappedIssue {
        context: &'static str,
        inner: GenericIssue<u8>,
    }

    #[derive(Debug, Error, Report)]
    #[aldebaran(crate)]
    enum EnumIssue {
        #[error("source issue")]
        #[report(title = "source issue", message = "invalid source")]
        Source(Span),

        #[error("wrapped issue")]
        #[report(title = "wrapped issue", message_with = issue_message)]
        Wrapped(GenericIssue<u8>),
    }

    #[test]
    fn generic_projected_message_and_inferred_target_are_preserved() {
        let issue = GenericIssue {
            target: Span::unit(3),
            value: 7_u8,
        };

        assert_eq!(Annotated::message(&issue), "expected a valid value");
        assert_eq!(Annotated::target(&issue), Span::unit(3));
        assert_eq!(ReportTrait::title(&issue), "invalid value");
        assert_eq!(ReportTrait::target(&issue), Some(Span::unit(3)));
    }

    #[test]
    fn stored_annotations_and_transparent_targets_are_preserved() {
        let labels = InlineAnnotations::new(Label::new("primary", Span::unit(5)), [Label::new("related", Span::unit(9))]);
        let issue = MultiIssue { labels };

        assert_eq!(*Annotated::message(&issue), "primary");
        assert_eq!(Annotated::target(&issue), Span::unit(5));

        let wrapped = WrappedIssue {
            context: "source",
            inner: GenericIssue {
                target: Span::unit(13),
                value: 1_u8,
            },
        };
        assert_eq!(Annotated::target(&wrapped), Span::unit(13));
        assert_eq!(ReportTrait::title(&wrapped), "invalid value");
        assert_eq!(wrapped.context, "source");
    }

    #[test]
    fn enum_alternatives_apply_their_own_report_policies() {
        let source = EnumIssue::Source(Span::unit(17));

        assert_eq!(Annotated::message(&source), "invalid source");
        assert_eq!(Annotated::target(&source), Span::unit(17));
        assert_eq!(ReportTrait::title(&source), "source issue");

        let wrapped = EnumIssue::Wrapped(GenericIssue {
            target: Span::unit(23),
            value: 9_u8,
        });

        assert_eq!(Annotated::message(&wrapped), "expected a valid value");
        assert_eq!(Annotated::target(&wrapped), Span::unit(23));
        assert_eq!(ReportTrait::title(&wrapped), "wrapped issue");
    }
}
