#![cfg_attr(not(test), no_std)]
#![doc = include_str!("../../../docs/part/COMPONENT.md")]
#![doc = include_str!("../README.md")]
#![doc = include_str!("../../../docs/part/FACADE.md")]
#![doc = include_str!("../../../docs/part/LICENSE.md")]

extern crate alloc;

pub mod recovery;

pub mod cursor;

pub mod builtin;

pub mod prelude {
    //! A prelude for the `aldebaran-text` crate.
    //!
    //! This simply re-exports the most commonly used items in the
    //! `aldebaran-text` crate.

    pub use crate::cursor::{Cursor, consume::Lex, error::Expected, internment::Internment, stream::LexStream};

    pub use crate::recovery::{AnyOf, Recover, Skip, Until};

    pub use crate::builtin::property::{
        AsciiAlphabetic, AsciiAlphanumeric, AsciiDigit, Digit, Eol, HorizontalWhitespace, NonEol, VerticalWhitespace,
    };
    pub use crate::builtin::{Ascii, Byte, Char};
}
