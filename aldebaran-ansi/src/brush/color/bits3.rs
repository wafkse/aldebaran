//! Standard and bright ANSI 3-bit terminal colors.
//!
//! [`Ansi3Bit`] stores foreground control codes and derives background codes by
//! the ANSI offset. The type also normalizes between bright and regular variants
//! and emits surface-aware terminal sequences.

use core::fmt;

use aldebaran_print::prelude::{Combine, Print};

use crate::{brush::style::TextAttribute, prelude::AnsiSequence};

use super::Surface;

/// A enumeration of 3-bit ANSI color codes.
///
/// The values of this enum correspond to the `foreground` color code, and since
/// all variants are unit variants, they can be cast to `u8` to get the actual
/// value.
///
/// To get the `background` color code, simply add 10 to the `foreground` color
/// code or make use of the [`Ansi3Bit::background`] method.
///
/// A method for the `foreground` color code is also provided,
/// [`Ansi3Bit::foreground`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Ansi3Bit {
    /// The color black.
    ///
    /// This is the #0th color in the ANSI color palette.
    ///
    /// Also represented with the (foreground code 30, background code 40)
    /// indices.
    Black = 30,

    /// The color red.
    ///
    /// This is the #1st color in the ANSI color palette.
    ///
    /// Also represented with the (foreground code 31, background code 41)
    /// indices.
    Red = 31,

    /// The color green.
    ///
    /// This is the #2nd color in the ANSI color palette.
    /// Also represented with the (foreground code 32, background code 42)
    /// indices.
    Green = 32,

    /// The color yellow.
    ///
    /// This is the #3rd color in the ANSI color palette.
    /// Also represented with the (foreground code 33, background code 43)
    /// indices.
    Yellow = 33,

    /// The color blue.
    ///
    /// This is the #4th color in the ANSI color palette.
    /// Also represented with the (foreground code 34, background code 44)
    /// indices.
    Blue = 34,

    /// The color magenta.
    ///
    /// This is the #5th color in the ANSI color palette.
    /// Also represented with the (foreground code 35, background code 45)
    /// indices.
    Magenta = 35,

    /// The color cyan.
    ///
    /// This is the #6th color in the ANSI color palette.
    ///
    /// Also represented with the (foreground code 36, background code 46)
    /// indices.
    Cyan = 36,

    /// The color white.
    ///
    /// This is the #7th color in the ANSI color palette.
    /// Also represented with the (foreground code 37, background code 47)
    /// indices.
    White = 37,

    /// The color bright black.
    ///
    /// This is a brightened version of the [`Ansi3Bit::Black`] color.
    ///
    /// This is represented with the (foreground code 90, background code 100)
    /// indices.
    BrightBlack = 90,
    /// The color bright red.
    ///
    /// This is a brightened version of the [`Ansi3Bit::Red`] color.
    ///
    /// This is represented with the (foreground code 91, background code 101)
    /// indices.
    BrightRed = 91,

    /// The color bright green.
    ///
    /// This is a brightened version of the [`Ansi3Bit::Green`] color.
    ///
    /// This is represented with the (foreground code 92, background code 102)
    /// indices.
    BrightGreen = 92,

    /// The color bright yellow.
    ///
    /// This is a brightened version of the [`Ansi3Bit::Yellow`] color.
    ///
    /// This is represented with the (foreground code 93, background code 103)
    /// indices.
    BrightYellow = 93,

    /// The color bright blue.
    ///
    /// This is a brightened version of the [`Ansi3Bit::Blue`] color.
    ///
    /// This is represented with the (foreground code 94, background code 104)
    /// indices.
    BrightBlue = 94,
    /// The color bright magenta.
    ///
    /// This is a brightened version of the [`Ansi3Bit::Magenta`] color.
    ///
    /// This is represented with the (foreground code 95, background code 105)
    /// indices.
    BrightMagenta = 95,

    /// The color bright cyan.
    ///
    /// This is a brightened version of the [`Ansi3Bit::Cyan`] color.
    ///
    /// This is represented with the (foreground code 96, background code 106)
    /// indices.
    BrightCyan = 96,

    /// The color bright white.
    ///
    /// This is a brightened version of the [`Ansi3Bit::White`] color.
    ///
    /// This is represented with the (foreground code 97, background code 107)
    /// indices.
    BrightWhite = 97,
}

impl Ansi3Bit {
    /// The offset between foreground and background ANSI 3-bit color codes.
    const FOREGROUND_TO_BACKGROUND_OFFSET: u8 = 10;

    /// Retrieve the `foreground` color code.
    #[inline]
    pub const fn foreground(self) -> u8 {
        self as u8
    }

    /// Retrieve the `background` color code.
    ///
    /// This is, effectively, the `foreground` color code plus a constant value
    /// of `10`.
    #[inline]
    pub const fn background(self) -> u8 {
        // NOTE: cannot ever panic, as 97 is the highest value for `Ansi3Bit`
        self as u8 + Self::FOREGROUND_TO_BACKGROUND_OFFSET
    }

    /// Retrieve the speficic 3-bit color code for the target [`Surface`].
    #[inline]
    pub const fn surface(self, surface: Surface) -> u8 {
        match surface {
            Surface::Foreground => self.foreground(),
            Surface::Background => self.background(),
        }
    }

    /// Convert the [`Ansi3Bit`] color to its bright variant.
    ///
    /// No-op for colors already deemed to be bright.
    #[inline]
    pub const fn bright(self) -> Self {
        // most compilers can optimize this to a MOVcc plus addition combo
        match self {
            Self::Black => Self::BrightBlack,
            Self::Red => Self::BrightRed,
            Self::Green => Self::BrightGreen,
            Self::Yellow => Self::BrightYellow,
            Self::Blue => Self::BrightBlue,
            Self::Magenta => Self::BrightMagenta,
            Self::Cyan => Self::BrightCyan,
            Self::White => Self::BrightWhite,
            _ => self,
        }
    }

    /// Normalize the [`Ansi3Bit`] color to its non-bright variant.
    ///
    /// No-op for colors already deemed to be non-bright.
    #[inline]
    pub const fn normal(self) -> Self {
        // most compilers can optimize this to a MOVcc plus substraction combo
        match self {
            Self::BrightBlack => Self::Black,
            Self::BrightRed => Self::Red,
            Self::BrightGreen => Self::Green,
            Self::BrightYellow => Self::Yellow,
            Self::BrightBlue => Self::Blue,
            Self::BrightMagenta => Self::Magenta,
            Self::BrightCyan => Self::Cyan,
            Self::BrightWhite => Self::White,
            _ => self,
        }
    }
}

impl AnsiSequence<Surface> for Ansi3Bit {
    fn emit_with_input<W>(&self, writer: &mut W, target_surface: Surface) -> fmt::Result
    where
        W: fmt::Write,
    {
        let target_code = self.surface(target_surface);

        '\x1B'
            .sequence('[')
            .sequence(TextAttribute::RESET_ALL_CODE)
            .sequence(';')
            .sequence(target_code)
            .sequence('m')
            .print(writer)
    }
}

impl AnsiSequence<(TextAttribute, Surface)> for Ansi3Bit {
    fn emit_with_input<W>(&self, writer: &mut W, (target_attribute, target_surface): (TextAttribute, Surface)) -> fmt::Result
    where
        W: fmt::Write,
    {
        let target_code = self.surface(target_surface);

        '\x1B'
            .sequence('[')
            .sequence(target_attribute.active_code())
            .sequence(';')
            .sequence(target_code)
            .sequence('m')
            .print(writer)
    }
}

impl AnsiSequence<(Option<TextAttribute>, Surface)> for Ansi3Bit {
    fn emit_with_input<W>(&self, writer: &mut W, (target_attribute, target_surface): (Option<TextAttribute>, Surface)) -> fmt::Result
    where
        W: fmt::Write,
    {
        let target_attribute = target_attribute
            .as_ref()
            .map(TextAttribute::active_code)
            .unwrap_or(TextAttribute::RESET_ALL_CODE);

        let target_code = self.surface(target_surface);

        '\x1B'
            .sequence('[')
            .sequence(target_attribute)
            .sequence(';')
            .sequence(target_code)
            .sequence('m')
            .print(writer)
    }
}

impl AnsiSequence<TextAttribute> for Ansi3Bit {
    fn emit_with_input<W>(&self, writer: &mut W, target_attr: TextAttribute) -> fmt::Result
    where
        W: fmt::Write,
    {
        let target_code = self.surface(Surface::default());

        '\x1B'
            .sequence('[')
            .sequence(target_attr.active_code())
            .sequence(';')
            .sequence(target_code)
            .sequence('m')
            .print(writer)
    }
}

impl AnsiSequence<Option<TextAttribute>> for Ansi3Bit {
    fn emit_with_input<W>(&self, writer: &mut W, mode: Option<TextAttribute>) -> fmt::Result
    where
        W: fmt::Write,
    {
        let target_attr = mode
            .as_ref()
            .map(TextAttribute::active_code)
            .unwrap_or(TextAttribute::RESET_ALL_CODE);

        let target_code = self.surface(Surface::default());

        '\x1B'
            .sequence('[')
            .sequence(target_attr)
            .sequence(';')
            .sequence(target_code)
            .sequence('m')
            .print(writer)
    }
}
