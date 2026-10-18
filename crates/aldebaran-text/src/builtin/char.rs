//! Unicode scalar equality for character sources. [`Char`] also accepts matching
//! ASCII bytes, letting the same predicate serve either source kind within ASCII.

use aldebaran_print::visualize::Visualize;
use core::{
    fmt,
    ops::{Deref, DerefMut},
};

use aldebaran_logic::{
    fmt::Precedence,
    prelude::{Assert, Formatter},
};

use aldebaran_print::prelude::Print;

/// Matches a Unicode scalar exactly, with the same rule for borrowed inputs.
/// Byte inputs match only in ASCII, so a multibyte UTF-8 encoding cannot satisfy this predicate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Char(pub char);

impl PartialEq<char> for Char {
    #[inline]
    fn eq(&self, other: &char) -> bool {
        let &Self(ref target_value) = self;

        target_value == other
    }
}

impl PartialEq<&char> for Char {
    #[inline]
    fn eq(&self, &other: &&char) -> bool {
        let &Self(ref target_value) = self;

        target_value == other
    }
}

impl Deref for Char {
    type Target = char;

    #[inline]
    fn deref(&self) -> &Self::Target {
        let &Self(ref target_value) = self;

        target_value
    }
}

impl DerefMut for Char {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        let &mut Self(ref mut target_value) = self;

        target_value
    }
}

impl From<char> for Char {
    #[inline]
    fn from(value: char) -> Self {
        Self(value)
    }
}

impl Assert<char> for Char {
    #[inline]
    fn assert(&self, input: char) -> bool {
        let &Self(target_value) = self;

        target_value == input
    }

    #[inline]
    fn output_with<W, F>(target_value: &Self, writer: &mut W, _: Precedence) -> fmt::Result
    where
        W: fmt::Write,
        F: Formatter,
    {
        Self::print(target_value, writer)
    }
}

impl Assert<&char> for Char {
    #[inline]
    fn assert(&self, input: &char) -> bool {
        let &Self(ref target_value) = self;

        target_value == input
    }

    #[inline]
    fn output_with<W, F>(target_value: &Self, writer: &mut W, _: Precedence) -> fmt::Result
    where
        W: fmt::Write,
        F: Formatter,
    {
        Self::print(target_value, writer)
    }
}

impl Assert<u8> for Char {
    #[inline]
    fn assert(&self, input: u8) -> bool {
        let &Self(target_value) = self;

        match input {
            0x00..0x80 => input as char == target_value,
            _ => false,
        }
    }

    #[inline]
    fn output_with<W, F>(target_value: &Self, writer: &mut W, _: Precedence) -> fmt::Result
    where
        W: fmt::Write,
        F: Formatter,
    {
        Self::print(target_value, writer)
    }
}

impl Assert<&u8> for Char {
    #[inline]
    fn assert(&self, &input: &u8) -> bool {
        <Self as Assert<u8>>::assert(self, input)
    }

    #[inline]
    fn output_with<W, F>(target_value: &Self, writer: &mut W, _: Precedence) -> fmt::Result
    where
        W: fmt::Write,
        F: Formatter,
    {
        Self::print(target_value, writer)
    }
}

impl Print for Char {
    type Context = ();
    #[inline]
    fn print_with_context<W>(&self, writer: &mut W, _: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        let &Self(ref target_value) = self;

        target_value.visualize(writer)
    }
}
