#![cfg_attr(not(test), no_std)]
#![doc = include_str!("../README.md")]

extern crate alloc;

pub mod recovery;

pub mod text;

pub mod basic;

pub mod storage;

pub mod prelude {
    //! A prelude for the `aldebaran-text` crate.
    //!
    //! This simply re-exports the most commonly used items in the
    //! `aldebaran-text` crate.

    pub use crate::text::{Text, assert::*, consume::Lex, internment::Internment, stream::LexStream};

    pub use crate::text::error::Expected;

    pub use crate::recovery::{AnyOf, Recover, Skip, Until};

    pub use crate::basic::{Ascii, Byte, Char};

    pub use crate::storage::*;
}
