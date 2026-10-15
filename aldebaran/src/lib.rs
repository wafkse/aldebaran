//! Aldebaran source-processing and diagnostic tooling.
//!
//! This facade groups the reusable compiler infrastructure crates in the
//! Aldebaran workspace. Language integrations remain separate consumers so this
//! crate stays focused on shared mechanisms.
#![no_std]

pub extern crate aldebaran_ansi as ansi;

pub extern crate aldebaran_dsa as dsa;

pub extern crate aldebaran_grammar as grammar;

pub extern crate aldebaran_hash as hash;

pub extern crate aldebaran_heap as heap;

pub extern crate aldebaran_ice as ice;

pub extern crate aldebaran_id as id;

pub extern crate aldebaran_interner as interner;

pub extern crate aldebaran_logic as logic;

pub extern crate aldebaran_primitive as primitive;

pub extern crate aldebaran_print as print;

pub extern crate aldebaran_report as report;

pub extern crate aldebaran_source as source;

pub extern crate aldebaran_span as span;

pub extern crate aldebaran_style as style;

pub extern crate aldebaran_text as text;

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
