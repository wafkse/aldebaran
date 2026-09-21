#![doc = include_str!("../README.md")]
#![no_std]
#![forbid(unsafe_code, missing_docs, rustdoc::all, clippy::all)]

pub mod assert;

pub mod choose;

pub mod input;

pub mod fmt;

pub mod adhoc;

pub mod connective;

pub mod select;

pub mod prelude {
    //! A prelude for the `aldebaran-logic` crate.
    //!
    //! This simply re-exports the most commonly used items in the
    //! `aldebaran-logic` crate.

    pub use crate::assert::{AsAssert, Assert, IntoAssert};
    pub use crate::choose::Choose;

    pub use crate::input::Input;

    pub use crate::fmt::{Formatter, common::DefaultFormatter};

    pub use crate::connective::*;

    pub use crate::select::Select;

    pub use aldebaran_logic_macro::{Assert, Choose};
}
