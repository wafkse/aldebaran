//! Byte equality across byte and character sources.
//!
//! See [`Byte`] for further information.

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

/// A singular byte.
///
/// This is a *new-type* over [`u8`] that represents a singular byte.
///
/// This can also be used as an assertion over character inputs, but only ASCII
/// values match across the two types. The same rules apply to borrowed inputs.
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

#[cfg(test)]
mod tests {
    use super::Byte;

    use aldebaran_logic::prelude::Assert;

    #[test]
    fn byte_assertion_matches_owned_and_borrowed_bytes() {
        let assertion = Byte(0xE9);
        let matching = 0xE9;
        let different = 0xE8;
        let matching_ref = &matching;
        let different_ref = &different;

        assert!(Assert::<u8>::assert(&assertion, matching));
        assert!(Assert::<&u8>::assert(&assertion, matching_ref));
        assert!(!Assert::<u8>::assert(&assertion, different));
        assert!(!Assert::<&u8>::assert(&assertion, different_ref));
    }

    #[test]
    fn character_assertion_matches_only_ascii_values() {
        let ascii_assertion = Byte(b'A');
        let matching_character = 'A';
        let different_character = 'a';
        let matching_character_ref = &matching_character;
        let different_character_ref = &different_character;

        assert!(Assert::<char>::assert(&ascii_assertion, matching_character));
        assert!(Assert::<&char>::assert(&ascii_assertion, matching_character_ref));
        assert!(!Assert::<char>::assert(&ascii_assertion, different_character));
        assert!(!Assert::<&char>::assert(&ascii_assertion, different_character_ref));

        let non_ascii_assertion = Byte(0xE9);
        let decoded_character = 'é';
        let decoded_character_ref = &decoded_character;

        assert!(!Assert::<char>::assert(&non_ascii_assertion, decoded_character));
        assert!(!Assert::<&char>::assert(&non_ascii_assertion, decoded_character_ref));
    }
}
