//! Color models used by ANSI painting and styling.
//!
//! [`Color`] unifies 3-bit palette colors, 8-bit indexed colors, and true-color
//! RGB values. [`Surface`] keeps foreground and background selection separate so
//! one color value can be emitted for either terminal surface.

pub mod bits3;
pub mod bits8;
pub mod rgb;

use bits3::Ansi3Bit;
use bits8::Ansi8Bit;
use rgb::AnsiRgb;

use core::fmt;

use aldebaran_print::prelude::{Combine, Print};

use crate::{prelude::AnsiSequence, reset::Resettable};

/// A color surface.
///
/// This is used to represent the actual subject to paint.
///
/// # Remarks
///
/// This *enum* implements [`Default`]. The default [`Surface`] is
/// [`Surface::Foreground`]. Albeit the rationale behind this may seem
/// arbitrary, it is based on the notion of the question "what color is that
/// `that`", where the answer is usually the `foreground` color of the text,
/// which, in most cases, is refered to implicitly, whereas the `background`
/// color would be refered to in a more explicit manner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Surface {
    /// The foreground color.
    #[default]
    Foreground,
    /// The background color.
    Background,
}

/// An enumeration that encompasses the various color models found in common
/// terminal emulators.
///
/// Albeit this is not an exhaustive list, it should cover the most common color
/// models.
///
/// ## ANSI Color Palette
///
/// This enumeration is independent and thus completely separate from the
/// semantics of the ANSI color palette. However, it does provide a way to
/// represent the colors in the ANSI color palette.
///
/// In other words, this makes no discernment between the foreground and
/// background colors, as the ANSI color palette does, but is rather a
/// representation of the colors themselves.
///
/// ## Exhaustiveness
///
/// This enumeration is not exhaustive, and may be extended in the future.
///
/// So, in the case of a match statement, it is recommended to use the `_` and
/// fall back to using the [`Default`] implementation of this `enum`.
///
/// ## Color Presets
///
/// This enum also provides a set of color presets as a set of associated
/// constants. These presets are based on the ANSI color palette with the
/// most widespread implementations in mind.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[non_exhaustive]
pub enum Color {
    /// ANSI 3-bit color.
    Bits3(Ansi3Bit),

    /// ANSI 8-bit color, or, in other words, a color from the 256-color
    /// palette.
    ///
    /// Note that [`Ansi8Bit`] is purely a newtype wrapper around `u8`, and
    Bits8(Ansi8Bit),

    /// A color that is determined by a RBG triplet.
    ///
    /// Otherwise known as a *true color*.
    Rgb(AnsiRgb),
}

impl Color {
    /// The color red.
    ///
    /// This is a preset for the color red. Same as
    /// `Color::Bits3(Ansi3Bit::Red)`.
    pub const RED: Self = Self::Bits3(Ansi3Bit::Red);

    /// The color green.
    ///
    /// This is a preset for the color green. Same as
    /// `Color::Bits3(Ansi3Bit::Green)`.
    pub const GREEN: Self = Self::Bits3(Ansi3Bit::Green);

    /// The color yellow.
    ///
    /// This is a preset for the color yellow. Same as
    /// `Color::Bits3(Ansi3Bit::Yellow)`.
    pub const YELLOW: Self = Self::Bits3(Ansi3Bit::Yellow);

    /// The color blue.
    ///
    /// This is a preset for the color blue. Same as
    /// `Color::Bits3(Ansi3Bit::Blue)`.
    pub const BLUE: Self = Self::Bits3(Ansi3Bit::Blue);

    /// The color magenta.
    ///
    /// This is a preset for the color magenta. Same as
    /// `Color::Bits3(Ansi3Bit::Magenta)`.
    pub const MAGENTA: Self = Self::Bits3(Ansi3Bit::Magenta);

    /// The color cyan.
    ///
    /// This is a preset for the color cyan. Same as
    /// `Color::Bits3(Ansi3Bit::Cyan)`.
    pub const CYAN: Self = Self::Bits3(Ansi3Bit::Cyan);

    /// The color white.
    ///
    /// This is a preset for the color white. Same as
    /// `Color::Bits3(Ansi3Bit::White)`.
    pub const WHITE: Self = Self::Bits3(Ansi3Bit::White);

    /// The color black.
    ///
    /// This is a preset for the color black. Same as
    /// `Color::Bits3(Ansi3Bit::Black)`.
    pub const BLACK: Self = Self::Bits3(Ansi3Bit::Black);

    /// The bright color red.
    ///
    /// This is a preset for the bright color red. Same as
    /// `Color::Bits3(Ansi3Bit::BrightRed)`.
    pub const BRIGHT_RED: Self = Self::Bits3(Ansi3Bit::BrightRed);

    /// The bright color green.
    ///
    /// This is a preset for the bright color green. Same as
    /// `Color::Bits3(Ansi3Bit::BrightGreen)`.
    pub const BRIGHT_GREEN: Self = Self::Bits3(Ansi3Bit::BrightGreen);

    /// The bright color yellow.
    ///
    /// This is a preset for the bright color yellow. Same as
    /// `Color::Bits3(Ansi3Bit::BrightYellow)`.
    pub const BRIGHT_YELLOW: Self = Self::Bits3(Ansi3Bit::BrightYellow);

    /// The bright color blue.
    ///
    /// This is a preset for the bright color blue. Same as
    /// `Color::Bits3(Ansi3Bit::BrightBlue)`.
    pub const BRIGHT_BLUE: Self = Self::Bits3(Ansi3Bit::BrightBlue);

    /// The bright color magenta.
    ///
    /// This is a preset for the bright color magenta. Same as
    /// `Color::Bits3(Ansi3Bit::BrightMagenta)`.
    pub const BRIGHT_MAGENTA: Self = Self::Bits3(Ansi3Bit::BrightMagenta);

    /// The bright color cyan.
    ///
    /// This is a preset for the bright color cyan. Same as
    /// `Color::Bits3(Ansi3Bit::BrightCyan)`.
    pub const BRIGHT_CYAN: Self = Self::Bits3(Ansi3Bit::BrightCyan);

    /// The bright color white.
    ///
    /// This is a preset for the bright color white. Same as
    /// `Color::Bits3(Ansi3Bit::BrightWhite)`.
    pub const BRIGHT_WHITE: Self = Self::Bits3(Ansi3Bit::BrightWhite);

    /// The bright color black.
    ///
    /// This is a preset for the bright color black. Same as
    /// `Color::Bits3(Ansi3Bit::BrightBlack)`.
    pub const BRIGHT_BLACK: Self = Self::Bits3(Ansi3Bit::BrightBlack);
}

impl Color {
    /// Determine if the current [`Color`] is a [`3-bit color`](Ansi3Bit).
    #[inline]
    pub const fn is_3bit(&self) -> bool {
        matches!(self, Self::Bits3(_))
    }

    /// Determine if the current [`Color`] is a [`8-bit color`](Ansi8Bit).
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
