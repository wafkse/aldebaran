//! Base and bright palette selection through SGR codes.
//!
//! [`Ansi3Bit`] identifies palette slots rather than fixed RGB values, so their
//! appearance follows the terminal palette. A [`Surface`] chooses whether
//! emission changes glyphs or their background.

use core::fmt;

use aldebaran_print::prelude::{Combine, Print};

use crate::{brush::style::TextAttribute, prelude::AnsiSequence};

use super::Surface;

/// The ANSI base and bright palette colors.
///
/// The discriminants are foreground SGR codes. Background codes are ten higher,
/// and [`surface`](Self::surface) selects the code for a [`Surface`]. Attribute
/// inputs can combine a color with an activation in the same sequence. An
/// absent optional attribute emits SGR 0 before the color, clearing prior SGR
/// settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Ansi3Bit {
    /// Black.
    Black = 30,

    /// Red.
    Red = 31,

    /// Green.
    Green = 32,

    /// Yellow.
    Yellow = 33,

    /// Blue.
    Blue = 34,

    /// Magenta.
    Magenta = 35,

    /// Cyan.
    Cyan = 36,

    /// White.
    White = 37,

    /// Bright black.
    BrightBlack = 90,
    /// Bright red.
    BrightRed = 91,

    /// Bright green.
    BrightGreen = 92,

    /// Bright yellow.
    BrightYellow = 93,

    /// Bright blue.
    BrightBlue = 94,
    /// Bright magenta.
    BrightMagenta = 95,

    /// Bright cyan.
    BrightCyan = 96,

    /// Bright white.
    BrightWhite = 97,
}

impl Ansi3Bit {
    /// The offset between foreground and background ANSI 3-bit color codes.
    const FOREGROUND_TO_BACKGROUND_OFFSET: u8 = 10;

    /// Palette SGR parameter for glyph color, without escape-sequence framing.
    #[inline]
    pub const fn foreground(self) -> u8 {
        self as u8
    }

    /// Palette SGR parameter for the glyph background, without sequence framing.
    #[inline]
    pub const fn background(self) -> u8 {
        // The largest foreground code is 97, so adding ten fits in `u8`.
        self as u8 + Self::FOREGROUND_TO_BACKGROUND_OFFSET
    }

    /// SGR palette parameter targeting the selected surface.
    /// This preserves the base or bright palette slot.
    #[inline]
    pub const fn surface(self, surface: Surface) -> u8 {
        match surface {
            Surface::Foreground => self.foreground(),
            Surface::Background => self.background(),
        }
    }

    /// Return the bright variant, preserving colors that are already bright.
    #[inline]
    pub const fn bright(self) -> Self {
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

    /// Return the base variant, preserving colors that are already base colors.
    #[inline]
    pub const fn normal(self) -> Self {
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

        '\x1B'.sequence('[').sequence(target_code).sequence('m').print(writer)
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
