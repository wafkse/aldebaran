#![cfg_attr(not(any(test, miri)), no_std)]
#![doc = include_str!("../README.md")]

extern crate alloc;

/// Reexports required by generated report code.
#[doc(hidden)]
pub mod reexport {
    pub use aldebaran_baseline::span::Span;
}

/// Rust code generation for diagnostic reports.
#[cfg(feature = "codegen")]
pub mod codegen {
    pub use aldebaran_report_macro::Report;
}

#[cfg(test)]
extern crate self as aldebaran_report;

#[cfg(test)]
mod tests {
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
        struct MultiIssue {
            labels: InlineAnnotations<Label<&'static str>, 1>,
        }

        #[derive(Debug, crate::codegen::Report)]
        #[error(transparent(inner))]
        #[report(transparent(inner))]
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
}

pub mod annotated;

pub mod attached;

pub mod report;

pub mod render;

pub mod prelude {
    //! A prelude for the `aldebaran-report` crate.
    //!
    //! This simply re-exports the most commonly used items in the
    //! `aldebaran-report` crate.

    pub use crate::{
        annotated::{Annotated, Annotations, InlineAnnotations, PrimaryAnnotations, label::Label, title::Title},
        attached::Attached,
        render::{
            Render, RenderMut,
            textual::{
                Present, Textual,
                offload::{
                    fancy::{Fancy, FancySettings, Fancyness},
                    hexdump::{Hexdump, HexdumpSettings},
                },
            },
        },
        report::{
            Report, SourceReport,
            kind::{ErrorKind, Simple},
            oneshot,
        },
    };
}
