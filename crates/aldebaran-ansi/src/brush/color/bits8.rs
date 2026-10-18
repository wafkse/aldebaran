//! ANSI 8-bit indexed terminal colors.
//!
//! [`Ansi8Bit`] stores one palette index and emits the corresponding foreground
//! or background sequence according to the requested surface. The representation
//! keeps terminal palette identity compact and independent from RGB conversion.

use core::fmt;

use aldebaran_print::prelude::{Combine, Print};

use crate::prelude::AnsiSequence;

use super::Surface;

/// A newtype wrapper around `u8` that represents an 8-bit ANSI color code.
///
/// This is equivalent to the 256-color palette.
///
/// Due to no discernable difference between the `foreground` and `background`
/// color for this palette, no accessor methods are provided alongside this
/// type.
///
/// ## Technical Details
///
/// The sequence for setting an 8-bit color as the foreground color is:
/// `ESC[38;5;{ID}m` where `{ID}` is the color code.
///
/// The sequence for setting an 8-bit color as the background color is:
/// `ESC[48;5;{ID}m` where `{ID}` is the color code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct Ansi8Bit(u8);

impl Ansi8Bit {
    /// Retrieve the inner [`u8`] value contained in this [`Ansi8Bit`].
    #[inline]
    pub const fn into_inner(self) -> u8 {
        let Self(inner) = self;

        inner
    }

    /// Create a new [`Ansi8Bit`] from the given `u8` value.
    #[inline]
    pub const fn from_u8(value: u8) -> Self {
        Self(value)
    }
}

impl AnsiSequence<Surface> for Ansi8Bit {
    fn emit_with_input<W>(&self, writer: &mut W, target_surface: Surface) -> fmt::Result
    where
        W: fmt::Write,
    {
        let surface_code = match target_surface {
            Surface::Foreground => 38,
            Surface::Background => 48,
        };

        let &Self(target_code) = self;

        '\x1B'
            .sequence('[')
            .sequence(surface_code)
            .sequence(';')
            .sequence(5)
            .sequence(';')
            .sequence(target_code)
            .sequence('m')
            .print(writer)
    }
}
