#![doc = include_str!("../README.md")]
#![no_std]

pub mod assert;

pub mod choose;

pub mod input;

pub mod fmt;

pub mod adhoc;

pub mod connective;

pub mod select;

pub mod builtin;

pub mod prelude {
    //! The `aldebaran-logic` prelude re-exports the most commonly used items for
    //! defining, composing, describing, and classifying assertions.

    pub use crate::assert::{AsAssert, Assert, IntoAssert};
    pub use crate::choose::Choose;

    pub use crate::input::Input;

    pub use crate::fmt::{Formatter, common::DefaultFormatter};

    pub use crate::connective::*;

    pub use crate::select::Select;

    pub use crate::builtin::Anything;

    pub use aldebaran_logic_macro::{Assert, Choose};
}
