//! Primitive assertion values used by text-processing algorithms.
//!
//! Byte and character wrappers preserve the exact input value while implementing
//! the logical assertion interfaces. ASCII and punctuation helpers provide
//! reusable semantic predicates for generic source and lexer code.

pub mod ascii;
pub mod byte;
pub mod char;

pub mod punct;

pub use self::{ascii::Ascii, byte::Byte, char::Char};
