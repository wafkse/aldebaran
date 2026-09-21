//! A crate for working with ASCII characters in a type-safe manner.
//!
//! This crate will remain in use until the `AsciiChar` type is stabilized in
//! the standard library.

use core::fmt;

use aldebaran_ice::Ice;
use aldebaran_logic::{
    fmt::Precedence,
    prelude::{Assert, Formatter, Input},
};

use aldebaran_print::prelude::Print;
use aldebaran_source::prelude::{AsciiByte, Source, SourceDissect, SourceIter, SourceLines, SourceMetadata};

use crate::{
    prelude::{Expected, Lex, Text},
    text::assert,
};

/// A type that is guaranteed to be an [`Assert`] for some ASCII character.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AlwaysAscii<const C: char>
where
    Self: Assert<char>;

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
                    #[doc = concat!("The `", stringify!($target_variant), "` `", stringify!($target_literal), "` ASCII character.\n")]
                    $(#[$variant_meta])*
                    $target_variant = const {
                        let target_value = $target_literal;

                        assert!(target_value.is_ascii(), "the character is not ASCII");

                        target_value as _
                    }
                ),*
            }

            impl $target_name {
                /// Create a new ASCII character from sole Unicode scalar value.
                ///
                /// # Panics
                ///
                /// Panics if the Unicode scalar value is not an ASCII character.
                #[inline]
                pub const fn or_panic(target_char: char) -> Self {
                    match target_char {
                        $(
                            $target_literal => Self::$target_variant,
                        )*
                        _ => Ice::<Self>::raise(),
                    }
                }

                /// Attempt to create a new ASCII character from sole Unicode scalar value.
                #[inline]
                pub const fn checked(target_char: char) -> Option<Self> {
                    match target_char {
                        $(
                            $target_literal => Some(Self::$target_variant),
                        )*
                        _ => None,
                    }
                }

                /// Determine the [`char`] for this ASCII character.
                #[inline]
                pub const fn as_char(&self) -> char {
                    *self as u8 as char
                }
            }

            $(

                impl<I> Assert<I> for AlwaysAscii<{ $target_literal }>
                where
                    I: Input,
                    Ascii: Assert<I>,
                {
                    #[inline]
                    fn assert(&self, input: I) -> bool {
                        let ref target_variant = Ascii::$target_variant;

                        Assert::<I>::assert(target_variant, input)
                    }

                    #[inline]
                    fn output_with<W, F>(_: &Self, writer: &mut W, precedence: Precedence) -> fmt::Result
                    where
                        W: fmt::Write,
                        F: Formatter,
                    {
                        let ref target_variant = Ascii::$target_variant;


                        Assert::<I>::output_with::<W, F>(target_variant, writer, precedence)
                    }
                }
            )*
    };
}

ascii!(
    /// A singular ASCII character in a text source.
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
    fn print_with_ctx<W>(&self, writer: &mut W, _: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        write!(writer, "{}", *self as u8 as char)
    }
}

impl<'source, S> Lex<'source, S> for Ascii
where
    S: Source<'source> + SourceDissect<'source> + SourceIter<'source> + SourceMetadata<'source> + SourceLines<'source> + ?Sized,
    assert::Ascii: Assert<S::Component>,
    Self: TryFrom<S::Component>,
{
    type Input<'input>
        = S::Component
    where
        'source: 'input;

    type Context = ();

    type Error
        = Expected<S::Component, assert::Ascii>
    where
        S: 'source;

    #[inline]
    fn pass<'input>(text: &'input mut Text<'source, S>) -> Result<Self::Input<'input>, Self::Error>
    where
        'source: 'input,
    {
        text.expect_is_next(assert::Ascii)
    }

    #[inline]
    fn build_with_ctx<'input>(input: Self::Input<'input>, _: &mut Self::Context) -> Result<Self, Self::Error>
    where
        'source: 'input,
        Self: Sized,
    {
        Ok(Ice::impossible(Self::try_from(input).ok()))
    }
}
