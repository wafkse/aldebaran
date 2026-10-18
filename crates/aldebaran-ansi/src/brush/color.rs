//! Color representations independent of their terminal surface.
//!
//! [`Color`] preserves a base or bright palette color, a 256-color index, or
//! RGB channels. Emission uses [`Surface`] to select foreground or background
//! codes without converting between those representations.

pub mod bits3;
pub mod bits8;
pub mod rgb;

use bits3::Ansi3Bit;
use bits8::Ansi8Bit;
use rgb::AnsiRgb;

use core::fmt;

use aldebaran_print::prelude::{Combine, Print};

use crate::{prelude::AnsiSequence, reset::Resettable};

/// Selects whether a color sequence affects the foreground or background.
///
/// The default surface is [`Foreground`](Self::Foreground).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Surface {
    /// Color of the displayed glyphs.
    #[default]
    Foreground,
    /// Color behind the displayed glyphs.
    Background,
}

/// A color value independent of foreground or background selection.
///
/// Emission preserves the variant's palette or RGB representation and uses a
/// [`Surface`] to select foreground or background output. Resetting any color
/// emits SGR 0, clearing text attributes and both color surfaces.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[non_exhaustive]
pub enum Color {
    /// A base or bright palette color.
    Bits3(Ansi3Bit),

    /// An index in the ANSI 256-color palette.
    Bits8(Ansi8Bit),

    /// A color specified by red, green, and blue channels.
    ///
    /// This is also called true color.
    Rgb(AnsiRgb),
}

impl Color {
    /// Base red from [`bits3::Ansi3Bit::Red`].
    pub const RED: Self = Self::Bits3(Ansi3Bit::Red);

    /// Base green from [`bits3::Ansi3Bit::Green`].
    pub const GREEN: Self = Self::Bits3(Ansi3Bit::Green);

    /// Base yellow from [`bits3::Ansi3Bit::Yellow`].
    pub const YELLOW: Self = Self::Bits3(Ansi3Bit::Yellow);

    /// Base blue from [`bits3::Ansi3Bit::Blue`].
    pub const BLUE: Self = Self::Bits3(Ansi3Bit::Blue);

    /// Base magenta from [`bits3::Ansi3Bit::Magenta`].
    pub const MAGENTA: Self = Self::Bits3(Ansi3Bit::Magenta);

    /// Base cyan from [`bits3::Ansi3Bit::Cyan`].
    pub const CYAN: Self = Self::Bits3(Ansi3Bit::Cyan);

    /// Base white from [`bits3::Ansi3Bit::White`].
    pub const WHITE: Self = Self::Bits3(Ansi3Bit::White);

    /// Base black from [`bits3::Ansi3Bit::Black`].
    pub const BLACK: Self = Self::Bits3(Ansi3Bit::Black);

    /// Bright red from [`bits3::Ansi3Bit::BrightRed`].
    pub const BRIGHT_RED: Self = Self::Bits3(Ansi3Bit::BrightRed);

    /// Bright green from [`bits3::Ansi3Bit::BrightGreen`].
    pub const BRIGHT_GREEN: Self = Self::Bits3(Ansi3Bit::BrightGreen);

    /// Bright yellow from [`bits3::Ansi3Bit::BrightYellow`].
    pub const BRIGHT_YELLOW: Self = Self::Bits3(Ansi3Bit::BrightYellow);

    /// Bright blue from [`bits3::Ansi3Bit::BrightBlue`].
    pub const BRIGHT_BLUE: Self = Self::Bits3(Ansi3Bit::BrightBlue);

    /// Bright magenta from [`bits3::Ansi3Bit::BrightMagenta`].
    pub const BRIGHT_MAGENTA: Self = Self::Bits3(Ansi3Bit::BrightMagenta);

    /// Bright cyan from [`bits3::Ansi3Bit::BrightCyan`].
    pub const BRIGHT_CYAN: Self = Self::Bits3(Ansi3Bit::BrightCyan);

    /// Bright white from [`bits3::Ansi3Bit::BrightWhite`].
    pub const BRIGHT_WHITE: Self = Self::Bits3(Ansi3Bit::BrightWhite);

    /// Bright black from [`bits3::Ansi3Bit::BrightBlack`].
    pub const BRIGHT_BLACK: Self = Self::Bits3(Ansi3Bit::BrightBlack);
}

impl Color {
    /// Whether emission uses the base or bright palette SGR codes.
    /// An indexed color with a visually similar entry does not satisfy this query.
    #[inline]
    pub const fn is_3bit(&self) -> bool {
        matches!(self, Self::Bits3(_))
    }

    /// Whether emission uses a 256-color palette index rather than palette SGR or RGB codes.
    #[inline]
    pub const fn is_8bit(&self) -> bool {
        matches!(self, Self::Bits8(_))
    }
}

impl AnsiSequence<Surface> for Color {
    #[inline]
    fn emit_with_input<W>(&self, writer: &mut W, surface: Surface) -> fmt::Result
    where
        W: fmt::Write,
    {
        match self {
            Self::Bits3(color) => color.emit_with_input(writer, surface),
            Self::Bits8(color) => color.emit_with_input(writer, surface),
            Self::Rgb(color) => color.emit_with_input(writer, surface),
        }
    }
}

impl<T> Resettable<T> for Color
where
    Self: AnsiSequence<T>,
{
    #[inline]
    fn reset<W>(&self, writer: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        '\x1B'.sequence('[').sequence('0').sequence('m').print(writer)
    }
}
