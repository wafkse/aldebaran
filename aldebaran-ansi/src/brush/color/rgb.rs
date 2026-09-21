//! ANSI true-color values represented as RGB channels.
//!
//! [`AnsiRgb`] retains the three channel bytes directly and emits the extended
//! ANSI sequence required by foreground or background surfaces. No palette
//! quantization is performed by this representation.

use core::fmt;

use aldebaran_print::prelude::{Combine, Print};

use crate::prelude::AnsiSequence;

use crate::prelude::Surface;

/// A RBG ANSI color determined by a RGB triplet.
///
/// This may also be refered to by the name of *true color*.
///
/// ## Technical Details
///
/// The sequence for setting a RGB color as the foreground color is:
/// `ESC[38;2;{R};{G};{B}m`
///
/// The sequence for setting a RGB color as the background color is:
/// `ESC[48;2;{R};{G};{B}m`
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct AnsiRgb([u8; 3]);

impl AnsiSequence<Surface> for AnsiRgb {
    fn emit_with_input<W>(&self, writer: &mut W, target_surface: Surface) -> fmt::Result
    where
        W: fmt::Write,
    {
        let &Self([r, g, b]) = self;

        let target_surface_code = match target_surface {
            Surface::Foreground => 38,
            Surface::Background => 48,
        };

        '\x1B'
            .sequence('[')
            .sequence(target_surface_code)
            .sequence(';')
            .sequence(2)
            .sequence(';')
            .sequence(r)
            .sequence(';')
            .sequence(g)
            .sequence(';')
            .sequence(b)
            .sequence('m')
            .print(writer)
    }
}

impl AnsiRgb {
    /// A black [`AnsiRgb`] color.
    pub const BLACK: Self = Self::array([0, 0, 0]);

    /// A red [`AnsiRgb`] color.
    pub const RED: Self = Self::array([255, 0, 0]);

    /// A green [`AnsiRgb`] color.
    pub const GREEN: Self = Self::array([0, 255, 0]);

    /// A yellow [`AnsiRgb`] color.
    pub const YELLOW: Self = Self::array([255, 255, 0]);

    /// A blue [`AnsiRgb`] color.
    pub const BLUE: Self = Self::array([0, 0, 255]);

    /// A magenta [`AnsiRgb`] color.
    pub const MAGENTA: Self = Self::array([255, 0, 255]);

    /// A cyan [`AnsiRgb`] color.
    pub const CYAN: Self = Self::array([0, 255, 255]);

    /// A white [`AnsiRgb`] color.
    pub const WHITE: Self = Self::array([255, 255, 255]);

    /// Instantiate a new [`AnsiRgb`] color from a 3-element array, in the [r,
    /// g, b] order.
    #[inline]
    pub const fn array([red, green, blue]: [u8; 3]) -> Self {
        Self([red, green, blue])
    }

    /// Instantiate a new [`AnsiRgb`] color from a 3-element tuple, in the (r,
    /// g, b) order.
    #[inline]
    pub const fn tuple((red, green, blue): (u8, u8, u8)) -> Self {
        Self::array([red, green, blue])
    }

    /// Attempt to instantiate a new [`AnsiRgb`] color from a slice of an
    /// arbitrary length.
    ///
    /// Will return `None` if the slice is not at least of length `3`.
    #[inline]
    pub const fn try_from_slice(slice: &[u8]) -> Option<Self> {
        // FIXME|HACK: remove this once the try trait can be used in const
        // contexts
        macro_rules! try_const {
            ($target_value:expr) => {
                match $target_value {
                    Some(value) => value,
                    None => return None,
                }
            };
        }

        let (&r, slice) = try_const!(slice.split_first());

        let (&g, slice) = try_const!(slice.split_first());

        let (&b, _) = try_const!(slice.split_first());

        Some(Self::tuple((r, g, b)))
    }

    /// Retrieve the red component of this [`AnsiRgb`] color.
    #[inline]
    pub const fn red(&self) -> u8 {
        let &Self([red, ..]) = self;

        red
    }

    /// Retrieve the green component of this [`AnsiRgb`] color.
    #[inline]
    pub const fn green(&self) -> u8 {
        let &Self([_, green, ..]) = self;

        green
    }

    /// Retrieve the blue component of this [`AnsiRgb`] color.
    #[inline]
    pub const fn blue(&self) -> u8 {
        let &Self([_, _, blue]) = self;

        blue
    }
}

impl Default for AnsiRgb {
    #[inline]
    fn default() -> Self {
        Self::WHITE
    }
}
