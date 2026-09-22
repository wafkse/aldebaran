//! Aldebaran source-processing and diagnostic tooling.
//!
//! This facade groups the reusable compiler infrastructure crates in the
//! Aldebaran workspace. Language integrations remain separate consumers so this
//! crate stays focused on shared mechanisms.

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
/// Data structures and algorithms used by compiler infrastructure.
pub extern crate aldebaran_dsa as dsa;
/// Token stream primitives for hand-written parsers.
pub extern crate aldebaran_grammar as grammar;
/// Hash implementation re-exports used throughout the workspace.
pub extern crate aldebaran_hash as hash;
/// Allocator and heap abstractions.
pub extern crate aldebaran_heap as heap;
/// Internal compiler error utilities.
pub extern crate aldebaran_ice as ice;
/// Typed identity collections and identifiers.
pub extern crate aldebaran_id as id;
/// Interning support for source-shaped values.
pub extern crate aldebaran_interner as interner;
/// Composable assertion logic used by source processing.
pub extern crate aldebaran_logic as logic;
/// Generic primitive-number traits and macros.
pub extern crate aldebaran_primitive as primitive;
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
/// Human-readable visualization of source-shaped values.
pub extern crate aldebaran_visualize as visualize;

pub mod prelude {
    //! A prelude for the `aldebaran` crate.
    //!
    //! This simply re-exports the most commonly used items in the
    //! `aldebaran` crate.

    pub use crate::ansi::prelude::*;
    pub use crate::dsa::prelude::*;
    pub use crate::grammar::prelude::*;
    pub use crate::heap::prelude::*;
    pub use crate::ice::Ice;
    pub use crate::id::prelude::*;
    pub use crate::interner::prelude::*;
    pub use crate::logic::prelude::*;
    pub use crate::primitive::prelude::*;
    pub use crate::print::prelude::*;
    pub use crate::report::prelude::*;
    pub use crate::source::prelude::*;
    pub use crate::span::prelude::*;
    pub use crate::style::prelude::*;
    pub use crate::text::prelude::*;
    pub use crate::visualize::prelude::*;

    #[cfg(feature = "report-codegen")]
    pub use crate::report::codegen::Report;
}
