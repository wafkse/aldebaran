//! Aldebaran source-processing and diagnostic tooling.
//!
//! This facade groups the stable source, text, logic, interning, styling, and
//! reporting crates used to build source-oriented tools. Language integrations
//! remain separate consumers so this crate stays focused on reusable mechanisms.

#![no_std]
#![forbid(
    unsafe_code,
    missing_docs,
    missing_debug_implementations,
    clippy::unwrap_used,
    clippy::std_instead_of_core
)]

/// ANSI terminal styling support.
pub extern crate aldebaran_ansi as ansi;
/// Interning support for source-shaped values.
pub extern crate aldebaran_interner as interner;
/// Composable assertion logic used by source processing.
pub extern crate aldebaran_logic as logic;
/// Structured printable values used by diagnostics.
pub extern crate aldebaran_print as print;
/// Renderer-independent diagnostics and built-in renderers.
pub extern crate aldebaran_report as report;
/// Source metadata and line handling.
pub extern crate aldebaran_source as source;
/// Source spans and insertion boundaries.
pub extern crate aldebaran_span as span;
/// Renderer and terminal presentation style metadata.
pub extern crate aldebaran_style as style;
/// Text traversal, lexing, and source-oriented matching.
pub extern crate aldebaran_text as text;

/// Common runtime imports for source-processing consumers.
pub mod prelude {
    //! A prelude for the `aldebaran` crate.
    //!
    //! This simply re-exports the most commonly used items in the
    //! `aldebaran` crate.

    pub use crate::report::prelude::{Annotated, Report, SourceReport};
    pub use crate::source::standard::Standard;
    pub use crate::span::prelude::*;
}
