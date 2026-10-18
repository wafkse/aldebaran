//! Lexical assertions for byte and character sources. [`Byte`] and [`Char`]
//! share ASCII equality across source kinds, [`Ascii`] names ASCII literals, and
//! [`property`] and [`punct`] describe character classes and delimiter alternatives.

pub mod ascii;
pub mod byte;
pub mod char;

pub mod property;
pub mod punct;

pub use self::{ascii::Ascii, byte::Byte, char::Char};
