#[cfg(feature = "codegen")]
mod derive_tests {

    use crate::{
        annotated::{Annotated, InlineAnnotations, label::Label},
        reexport::Span,
        report::Report as ReportTrait,
    };

    #[derive(Debug, crate::codegen::Report)]
    #[error("invalid value {value}")]
    #[report(title = "invalid value", message_with = issue_message)]
    // NOTE(invariant): None.
    struct GenericIssue<T> {
        target: Span,
        value: T,
    }

    fn issue_message<T>(_: &GenericIssue<T>) -> &'static str {
        "expected a valid value"
    }

    #[derive(Debug, crate::codegen::Report)]
    #[error("multiple labels")]
    #[report(title = "multiple labels", annotations = labels)]
    // NOTE(invariant): None.
    struct MultiIssue {
        labels: InlineAnnotations<Label<&'static str>, 1>,
    }

    #[derive(Debug, crate::codegen::Report)]
    #[error(transparent(inner))]
    #[report(transparent(inner))]
    // NOTE(invariant): None.
    struct WrappedIssue {
        context: &'static str,
        inner: GenericIssue<u8>,
    }

    #[derive(Debug, crate::codegen::Report)]
    enum EnumIssue {
        #[error("source issue")]
        #[report(title = "source issue", message = "invalid source")]
        Source(Span),

        #[error("wrapped issue")]
        #[report(title = "wrapped issue", message_with = issue_message)]
        Wrapped(GenericIssue<u8>),
    }

    #[test]
    fn derives_generic_report_with_projected_message_and_inferred_target() {
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
    fn derives_stored_annotations_and_explicit_transparent_target() {
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
    fn derives_each_enum_alternative_policy() {
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
