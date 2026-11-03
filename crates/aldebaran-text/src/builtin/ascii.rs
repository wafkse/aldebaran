//! Named ASCII values usable as predicates over bytes and Unicode scalars.
//!
//! See [`Ascii`] and [`Literal`] for further information.

use core::fmt;

use aldebaran_ice::Ice;
use aldebaran_logic::{
    fmt::Precedence,
    prelude::{Assert, Formatter, Input},
};

use aldebaran_print::prelude::Print;
use aldebaran_source::prelude::{AsciiByte, Source, SourceDissect, SourceIter, SourceLines, SourceMetadata};

use crate::{
    builtin::property,
    prelude::{Cursor, Expected, Lex},
};

/// A type that is guaranteed to be an [`Assert`] for some ASCII character.
///
/// The character is given by the constant `C`, so it is fixed at compile time.
/// Only ASCII values have assertion implementations, which match that character
/// in either byte or character inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Literal<const C: char>
where
    Self: Assert<char>;

/// Named ASCII values and compile-time literals must denote the same character.
macro_rules! ascii {
    () => {};
    (
        $(
            #[$target_meta:meta]
        )*
        $target_vis:vis enum $target_name:ident {
            $(
                $(#[$variant_meta:meta])*
                $target_variant:ident = $target_literal:literal
            ),*

            $(,)?
        }
    ) => {
            $(
                #[$target_meta]
            )*
            $target_vis enum $target_name {
                $(
                    #[doc = concat!("Matches ASCII character ``", stringify!($target_literal), "``.\n")]
                    $(#[$variant_meta])*
                    $target_variant = const {
                        let target_value = $target_literal;

                        assert!(target_value.is_ascii(), "the character is not ASCII");

                        target_value as isize
                    }
                ),*
            }

            impl $target_name {
                /// Require `target_char` to denote an ASCII character.
                ///
                /// # Panics
                ///
                /// Panics if `target_char` is outside ASCII.
                #[inline]
                pub const fn or_panic(target_char: char) -> Self {
                    match target_char {
                        $(
                            $target_literal => Self::$target_variant,
                        )*
                        _ => Ice::<Self>::raise(),
                    }
                }

                /// Reject non-ASCII scalars rather than interpreting their UTF-8 encoding.
                #[inline]
                pub const fn checked(target_char: char) -> Option<Self> {
                    match target_char {
                        $(
                            $target_literal => Some(Self::$target_variant),
                        )*
                        _ => None,
                    }
                }

                /// The same ASCII code point as a Unicode scalar, without decoding or replacement.
                #[inline]
                pub const fn as_char(&self) -> char {
                    *self as u8 as char
                }
            }

            $(

                impl<I> Assert<I> for Literal<{ $target_literal }>
                where
                    I: Input,
                    Ascii: Assert<I>,
                {
                    #[inline]
                    fn assert(&self, input: I) -> bool {
                        let target_variant = &Ascii::$target_variant;

                        Assert::<I>::assert(target_variant, input)
                    }

                    #[inline]
                    fn output_with<W, F>(_: &Self, writer: &mut W, precedence: Precedence) -> fmt::Result
                    where
                        W: fmt::Write,
                        F: Formatter,
                    {
                        let target_variant = &Ascii::$target_variant;


                        Assert::<I>::output_with::<W, F>(target_variant, writer, precedence)
                    }
                }
            )*
    };
}

ascii!(
    /// A singular ASCII character in a text source.
    ///
    /// Each variant can be used as an assertion over bytes or characters, including
    /// ASCII control characters. When used as a lexer, this type consumes the next
    /// component even if it is not ASCII and recognition therefore fails.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum Ascii {
        Null = '\x00',
        StartOfHeading = '\x01',
        StartOfText = '\x02',
        EndOfText = '\x03',
        EndOfTransmission = '\x04',
        Enquiry = '\x05',
        Acknowledge = '\x06',
        Bell = '\x07',
        Backspace = '\x08',
        Tab = '\x09',
        LineFeed = '\x0A',
        VerticalTab = '\x0B',
        FormFeed = '\x0C',
        CarriageReturn = '\x0D',
        ShiftOut = '\x0E',
        ShiftIn = '\x0F',
        DataLinkEscape = '\x10',
        DeviceControl1 = '\x11',
        DeviceControl2 = '\x12',
        DeviceControl3 = '\x13',
        DeviceControl4 = '\x14',
        NegativeAcknowledge = '\x15',
        SynchronousIdle = '\x16',
        EndOfBlock = '\x17',
        Cancel = '\x18',
        EndOfMedium = '\x19',
        Substitute = '\x1A',
        Escape = '\x1B',
        FileSeparator = '\x1C',
        GroupSeparator = '\x1D',
        RecordSeparator = '\x1E',
        UnitSeparator = '\x1F',
        Space = ' ',
        Exclamation = '!',
        Quote = '"',
        Hash = '#',
        Dollar = '$',
        Percent = '%',
        Ampersand = '&',
        Apostrophe = '\'',
        LeftParenthesis = '(',
        RightParenthesis = ')',
        Asterisk = '*',
        Plus = '+',
        Comma = ',',
        Minus = '-',
        Period = '.',
        Slash = '/',
        Digit0 = '0',
        Digit1 = '1',
        Digit2 = '2',
        Digit3 = '3',
        Digit4 = '4',
        Digit5 = '5',
        Digit6 = '6',
        Digit7 = '7',
        Digit8 = '8',
        Digit9 = '9',
        Colon = ':',
        Semicolon = ';',
        LessThan = '<',
        Equals = '=',
        GreaterThan = '>',
        Question = '?',
        At = '@',
        UpperA = 'A',
        UpperB = 'B',
        UpperC = 'C',
        UpperD = 'D',
        UpperE = 'E',
        UpperF = 'F',
        UpperG = 'G',
        UpperH = 'H',
        UpperI = 'I',
        UpperJ = 'J',
        UpperK = 'K',
        UpperL = 'L',
        UpperM = 'M',
        UpperN = 'N',
        UpperO = 'O',
        UpperP = 'P',
        UpperQ = 'Q',
        UpperR = 'R',
        UpperS = 'S',
        UpperT = 'T',
        UpperU = 'U',
        UpperV = 'V',
        UpperW = 'W',
        UpperX = 'X',
        UpperY = 'Y',
        UpperZ = 'Z',
        LeftBracket = '[',
        Backslash = '\\',
        RightBracket = ']',
        Caret = '^',
        Underscore = '_',
        Backtick = '`',
        LowerA = 'a',
        LowerB = 'b',
        LowerC = 'c',
        LowerD = 'd',
        LowerE = 'e',
        LowerF = 'f',
        LowerG = 'g',
        LowerH = 'h',
        LowerI = 'i',
        LowerJ = 'j',
        LowerK = 'k',
        LowerL = 'l',
        LowerM = 'm',
        LowerN = 'n',
        LowerO = 'o',
        LowerP = 'p',
        LowerQ = 'q',
        LowerR = 'r',
        LowerS = 's',
        LowerT = 't',
        LowerU = 'u',
        LowerV = 'v',
        LowerW = 'w',
        LowerX = 'x',
        LowerY = 'y',
        LowerZ = 'z',
        LeftBrace = '{',
        Pipe = '|',
        RightBrace = '}',
        Tilde = '~',
        Delete = '\x7F',
    }
);

impl From<AsciiByte> for Ascii {
    #[inline]
    fn from(value: AsciiByte) -> Self {
        Self::or_panic(value.as_char())
    }
}

impl TryFrom<char> for Ascii {
    type Error = ();

    #[inline]
    fn try_from(value: char) -> Result<Self, Self::Error> {
        Self::checked(value).ok_or(())
    }
}

impl TryFrom<&char> for Ascii {
    type Error = ();

    #[inline]
    fn try_from(value: &char) -> Result<Self, Self::Error> {
        Self::checked(*value).ok_or(())
    }
}

impl TryFrom<u8> for Ascii {
    type Error = ();

    #[inline]
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::checked(value as char).ok_or(())
    }
}

impl TryFrom<&u8> for Ascii {
    type Error = ();

    #[inline]
    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        Self::checked(*value as char).ok_or(())
    }
}

impl Assert<u8> for Ascii {
    #[inline]
    fn assert(&self, input: u8) -> bool {
        *self as u8 == input
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

impl Assert<&u8> for Ascii {
    #[inline]
    fn assert(&self, input: &u8) -> bool {
        *self as u8 == *input
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

impl Assert<char> for Ascii {
    #[inline]
    fn assert(&self, input: char) -> bool {
        *self as u8 as char == input
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

impl Assert<&char> for Ascii {
    #[inline]
    fn assert(&self, input: &char) -> bool {
        *self as u8 as char == *input
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

impl Print for Ascii {
    type Context = ();
    #[inline]
    fn print_with_context<W>(&self, writer: &mut W, _: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        write!(writer, "{}", *self as u8 as char)
    }
}

impl<'source, S> Lex<'source, S> for Ascii
where
    S: Source<'source> + SourceDissect<'source> + SourceIter<'source> + SourceMetadata<'source> + SourceLines<'source> + ?Sized,
    property::Ascii: Assert<S::Component>,
    Self: TryFrom<S::Component>,
{
    type Input<'input>
        = S::Component
    where
        'source: 'input;

    type Context = ();

    type Error
        = Expected<S::Component, property::Ascii>
    where
        S: 'source;

    #[inline]
    fn pass<'input>(text: &'input mut Cursor<'source, S>) -> Result<Self::Input<'input>, Self::Error>
    where
        'source: 'input,
    {
        text.expect_is_next(property::Ascii)
    }

    #[inline]
    fn build_with_context<'input>(input: Self::Input<'input>, _: &mut Self::Context) -> Result<Self, Self::Error>
    where
        'source: 'input,
        Self: Sized,
    {
        Ok(Ice::impossible(Self::try_from(input).ok()))
    }
}

#[cfg(test)]
mod tests {
    use super::{Ascii, Literal};

    use alloc::string::String;

    use aldebaran_logic::{fmt::common::Human, prelude::Assert};

    #[test]
    fn ascii_assertion_matches_every_ascii_byte_and_character_form() {
        for target_byte in 0_u8..=0x7F {
            let target_char = char::from(target_byte);
            let assertion = Ascii::or_panic(target_char);
            let mismatching_byte = target_byte | 0x80;
            let target_byte_ref = &target_byte;
            let target_char_ref = &target_char;
            let mismatching_byte_ref = &mismatching_byte;
            let mismatching_char = if target_byte == 0x7F {
                '\u{80}'
            } else {
                char::from(target_byte + 1)
            };
            let mismatching_char_ref = &mismatching_char;

            assert!(Assert::<u8>::assert(&assertion, target_byte));
            assert!(Assert::<&u8>::assert(&assertion, target_byte_ref));
            assert!(Assert::<char>::assert(&assertion, target_char));
            assert!(Assert::<&char>::assert(&assertion, target_char_ref));

            assert!(!Assert::<u8>::assert(&assertion, mismatching_byte));
            assert!(!Assert::<&u8>::assert(&assertion, mismatching_byte_ref));
            assert!(!Assert::<char>::assert(&assertion, mismatching_char));
            assert!(!Assert::<&char>::assert(&assertion, mismatching_char_ref));
        }
    }

    #[test]
    fn checked_and_try_from_reject_non_ascii_values() {
        assert_eq!(Ascii::checked('\0'), Some(Ascii::Null));
        assert_eq!(Ascii::checked('\u{7F}'), Some(Ascii::Delete));
        assert_eq!(Ascii::checked('\u{80}'), None);
        assert_eq!(Ascii::try_from(0_u8), Ok(Ascii::Null));
        assert_eq!(Ascii::try_from(0x7F_u8), Ok(Ascii::Delete));
        assert_eq!(Ascii::try_from(0x80_u8), Err(()));
    }

    #[test]
    fn literal_assertion_matches_owned_and_borrowed_inputs() {
        let assertion = Literal::<'A'>;
        let matching_byte = b'A';
        let matching_char = 'A';
        let matching_byte_ref = &matching_byte;
        let matching_char_ref = &matching_char;
        let different_byte = b'a';
        let different_byte_ref = &different_byte;
        let different_char = 'é';
        let different_char_ref = &different_char;

        assert!(Assert::<u8>::assert(&assertion, matching_byte));
        assert!(Assert::<&u8>::assert(&assertion, matching_byte_ref));
        assert!(Assert::<char>::assert(&assertion, matching_char));
        assert!(Assert::<&char>::assert(&assertion, matching_char_ref));

        assert!(!Assert::<u8>::assert(&assertion, different_byte));
        assert!(!Assert::<&u8>::assert(&assertion, different_byte_ref));
        assert!(!Assert::<char>::assert(&assertion, different_char));
        assert!(!Assert::<&char>::assert(&assertion, different_char_ref));

        let mut output = String::new();
        let result = Assert::<char>::output::<_, Human>(&assertion, &mut output);

        assert!(result.is_ok());
        assert_eq!(output, "A");
    }
}
