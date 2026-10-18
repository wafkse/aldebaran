#![cfg_attr(not(any(test, miri)), no_std)]
#![doc = include_str!("../README.md")]

extern crate alloc;

/// Reexports required by generated report code.
#[doc(hidden)]
pub mod reexport {
    pub use aldebaran_baseline::span::Span;
}

/// Derive support for diagnostic report types.
#[cfg(feature = "codegen")]
pub mod codegen {
    pub use aldebaran_report_macro::Report;
}

#[cfg(test)]
extern crate self as aldebaran_report;

#[cfg(test)]
mod tests;

pub mod annotated;

pub mod attached;

pub mod report;

pub mod render;

pub mod prelude {
    //! The report crate prelude re-exports its commonly used diagnostic, annotation,
    //! and rendering contracts.

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
