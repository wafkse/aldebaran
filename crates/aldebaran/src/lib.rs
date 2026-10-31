#![doc = include_str!("../README.md")]
#![doc = include_str!("../../../docs/part/LICENSE.md")]
#![no_std]

pub use aldebaran_ansi as ansi;

pub use aldebaran_baseline as baseline;

pub use aldebaran_grammar as grammar;

pub use aldebaran_ice as ice;

pub use aldebaran_id as id;

pub use aldebaran_interner as interner;

pub use aldebaran_logic as logic;

pub use aldebaran_print as print;

pub use aldebaran_report as report;

pub use aldebaran_source as source;

pub use aldebaran_text as text;

#[doc(hidden)]
pub mod rexport {
    //! Internal crates exposed under their original names for derive macro expansions.
    //!
    //! These paths let generated implementations use the facade as their sole
    //! Aldebaran dependency.

    /// ANSI terminal styling primitives.
    pub use aldebaran_ansi;

    /// Source coordinates and reusable type-level utilities.
    pub use aldebaran_baseline;

    /// Token stream primitives for hand-written parsers.
    pub use aldebaran_grammar;

    /// Internal compiler error utilities.
    pub use aldebaran_ice;

    /// Strongly typed identities and ID-indexed collections.
    pub use aldebaran_id;

    /// Borrowed and owned value interning with stable identities.
    pub use aldebaran_interner;

    /// Composable assertions, predicates, and input classification.
    pub use aldebaran_logic;

    /// Sink-oriented printing with optional external context.
    pub use aldebaran_print;

    /// Diagnostic reports and source annotations.
    pub use aldebaran_report;

    /// Source capabilities and line-oriented metadata.
    pub use aldebaran_source;

    /// Lexical scanning and text processing.
    pub use aldebaran_text;
}

pub mod prelude {
    //! The `aldebaran` prelude.
    //!
    //! Re-exports the most commonly used items from this crate.

    pub use crate::ansi::prelude::*;

    pub use crate::baseline::prelude::*;

    pub use crate::grammar::prelude::*;

    pub use crate::ice::Ice;

    pub use crate::id::prelude::*;

    pub use crate::interner::prelude::*;

    pub use crate::logic::prelude::*;

    pub use crate::print::prelude::*;

    pub use crate::report::prelude::*;

    pub use crate::source::prelude::*;

    pub use crate::text::prelude::*;

    #[cfg(feature = "report-codegen")]
    pub use crate::report::codegen::Report;
}
