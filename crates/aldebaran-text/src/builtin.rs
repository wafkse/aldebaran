//! Primitive assertion values used by text-processing algorithms.
//!
//! See [`Byte`], [`Char`], and [`Ascii`] for further information.

pub mod ascii;
pub mod byte;
pub mod char;

pub mod property;
pub mod punct;

pub use self::{ascii::Ascii, byte::Byte, char::Char};
