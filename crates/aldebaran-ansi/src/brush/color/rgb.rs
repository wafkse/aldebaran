//! Explicit color channels for terminal true-color output.
//!
//! [`AnsiRgb`] emits red, green, and blue bytes without palette quantization.
//! [`Surface`] selects foreground or background emission without changing
//! the channel values.

use core::fmt;

use aldebaran_print::prelude::{Combine, Print};

use crate::prelude::AnsiSequence;

use crate::prelude::Surface;

/// An ANSI true-color value containing red, green, and blue channel bytes.
///
/// Channels range from zero intensity to 255 and are emitted without palette
/// conversion. The default value is [`AnsiRgb::WHITE`].
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
// NOTE(invariant): None.
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

    /// Construct a color from channel bytes in red, green, blue order.
    #[inline]
    pub const fn array([red, green, blue]: [u8; 3]) -> Self {
        Self([red, green, blue])
    }

    /// Construct a color from a tuple in red, green, blue order.
    #[inline]
    pub const fn tuple((red, green, blue): (u8, u8, u8)) -> Self {
        Self::array([red, green, blue])
    }

    /// Read the first three channel bytes from `slice`.
    ///
    /// Fewer than three bytes yield `None`. The bytes are interpreted in red,
    /// green, blue order and any remaining bytes are ignored.
    #[inline]
    pub const fn try_from_slice(slice: &[u8]) -> Option<Self> {
        // Const evaluation requires explicit option propagation here.
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

    /// Red intensity from 0 through 255 in the emitted true-color value.
    #[inline]
    pub const fn red(&self) -> u8 {
        let &Self([red, ..]) = self;

        red
    }

    /// Green intensity from 0 through 255 in the emitted true-color value.
    #[inline]
    pub const fn green(&self) -> u8 {
        let &Self([_, green, ..]) = self;

        green
    }

    /// Blue intensity from 0 through 255 in the emitted true-color value.
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
