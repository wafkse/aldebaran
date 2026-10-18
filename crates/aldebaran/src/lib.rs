#![doc = include_str!("../README.md")]
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
