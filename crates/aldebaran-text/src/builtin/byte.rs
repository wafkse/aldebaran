//! Byte equality across byte and character sources. [`Byte`] accepts any byte
//! value but restricts cross-kind matching to ASCII.

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

/// Matches a byte exactly, with the same rule for borrowed inputs.
/// Character inputs match only in ASCII, so byte values above `0x7f` never match a character.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Byte(pub u8);

impl PartialEq<u8> for Byte {
    #[inline]
    fn eq(&self, other: &u8) -> bool {
        let &Self(ref target_value) = self;

        target_value == other
    }
}

impl PartialEq<&u8> for Byte {
    #[inline]
    fn eq(&self, &other: &&u8) -> bool {
        let &Self(ref target_value) = self;

        target_value == other
    }
}

impl PartialEq<char> for Byte {
    #[inline]
    fn eq(&self, &other: &char) -> bool {
        let &Self(target_value) = self;

        match other {
            '\x00'..='\x7f' => target_value as char == other,
            _ => false,
        }
    }
}

impl PartialEq<&char> for Byte {
    #[inline]
    fn eq(&self, &other: &&char) -> bool {
        <Self as PartialEq<char>>::eq(self, other)
    }
}

impl Deref for Byte {
    type Target = u8;

    #[inline]
    fn deref(&self) -> &Self::Target {
        let &Self(ref target_value) = self;

        target_value
    }
}

impl DerefMut for Byte {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        let &mut Self(ref mut target_value) = self;

        target_value
    }
}

impl From<u8> for Byte {
    #[inline]
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl Assert<u8> for Byte {
    #[inline]
    fn assert(&self, input: u8) -> bool {
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

impl Assert<&u8> for Byte {
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

impl Assert<char> for Byte {
    #[inline]
    fn assert(&self, input: char) -> bool {
        let &Self(target_value) = self;

        match input {
            '\u{00}'..'\u{80}' => target_value as char == input,
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

impl Assert<&char> for Byte {
    #[inline]
    fn assert(&self, &input: &char) -> bool {
        <Self as Assert<char>>::assert(self, input)
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

impl Print for Byte {
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
