//! Aldebaran source-processing and diagnostic tooling.
//!
//! This facade groups the reusable compiler infrastructure crates in the
//! Aldebaran workspace. Language integrations remain separate consumers so this
//! crate stays focused on shared mechanisms.
#![no_std]

pub use aldebaran_ansi as ansi;

pub use aldebaran_dsa as dsa;

pub use aldebaran_grammar as grammar;

pub use aldebaran_hash as hash;

pub use aldebaran_heap as heap;

pub use aldebaran_ice as ice;

pub use aldebaran_id as id;

pub use aldebaran_interner as interner;

pub use aldebaran_logic as logic;

pub use aldebaran_primitive as primitive;

pub use aldebaran_print as print;

pub use aldebaran_report as report;

pub use aldebaran_source as source;

pub use aldebaran_span as span;

pub use aldebaran_style as style;

pub use aldebaran_text as text;

pub use aldebaran_visualize as visualize;

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
