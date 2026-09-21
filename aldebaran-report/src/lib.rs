#![cfg_attr(not(any(test, miri)), no_std)]
#![forbid(
    missing_docs,
    missing_debug_implementations,
    clippy::missing_panics_doc,
    clippy::missing_safety_doc,
    clippy::missing_transmute_annotations,
    clippy::unwrap_used
)]
#![doc = include_str!("../README.md")]

extern crate alloc;

/// Reexports required by generated report code.
#[doc(hidden)]
pub mod reexport {
    pub use aldebaran_span::span::Span;
}

/// Rust code generation for diagnostic reports.
#[cfg(feature = "codegen")]
pub mod codegen {
    pub use aldebaran_report_macro::Report;
}

#[cfg(test)]
extern crate self as aldebaran_report;

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
