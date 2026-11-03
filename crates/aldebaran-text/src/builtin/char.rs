//! Unicode scalar equality for character sources.
//!
//! See [`Char`] for further information.

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

/// A singular character.
///
/// This is a *newtype* over [`char`] that represents a singular character.
///
/// This can also be used as an assertion over byte inputs, but only ASCII
/// values match across the two types. The same rules apply to borrowed inputs.
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

#[cfg(test)]
mod tests {
    use super::Char;

    use aldebaran_logic::prelude::Assert;

    #[test]
    fn character_assertion_matches_owned_and_borrowed_scalars() {
        let assertion = Char('é');
        let matching = 'é';
        let different = 'e';
        let matching_ref = &matching;
        let different_ref = &different;

        assert!(Assert::<char>::assert(&assertion, matching));
        assert!(Assert::<&char>::assert(&assertion, matching_ref));
        assert!(!Assert::<char>::assert(&assertion, different));
        assert!(!Assert::<&char>::assert(&assertion, different_ref));
    }

    #[test]
    fn byte_assertion_matches_only_ascii_values() {
        let ascii_assertion = Char('A');
        let matching_byte = b'A';
        let different_byte = b'a';
        let matching_byte_ref = &matching_byte;
        let different_byte_ref = &different_byte;

        assert!(Assert::<u8>::assert(&ascii_assertion, matching_byte));
        assert!(Assert::<&u8>::assert(&ascii_assertion, matching_byte_ref));
        assert!(!Assert::<u8>::assert(&ascii_assertion, different_byte));
        assert!(!Assert::<&u8>::assert(&ascii_assertion, different_byte_ref));

        let non_ascii_assertion = Char('é');
        let encoded_byte = 0xE9;
        let encoded_byte_ref = &encoded_byte;

        assert!(!Assert::<u8>::assert(&non_ascii_assertion, encoded_byte));
        assert!(!Assert::<&u8>::assert(&non_ascii_assertion, encoded_byte_ref));
    }
}
