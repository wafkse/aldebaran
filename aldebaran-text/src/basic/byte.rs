//! Byte-oriented assertion values for text processing.
//!
//! [`Byte`] wraps one `u8` and participates in assertion, printing, and ASCII
//! compatible comparisons. It lets generic text algorithms treat a concrete byte
//! value as a reusable predicate without erasing the underlying byte identity.

use aldebaran_visualize::visual::Visualize;
use core::{
    fmt,
    ops::{Deref, DerefMut},
};

use aldebaran_logic::{
    fmt::Precedence,
    prelude::{Assert, Formatter},
};

use aldebaran_print::prelude::Print;

/// A singular byte.
///
/// This is a *new-type* over [`u8`] that represents a singular byte.
///
/// Be noted that this is equatable to [`crate::basic::Char`], as per the Unicode standard.
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
    fn print_with_ctx<W>(&self, writer: &mut W, _: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        let &Self(ref target_value) = self;

        target_value.visualize(writer)
    }
}
