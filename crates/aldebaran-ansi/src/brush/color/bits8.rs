//! Indexed terminal colors selected without palette conversion.
//!
//! [`Ansi8Bit`] emits its index through the extended SGR color form, with
//! [`Surface`] choosing foreground or background.
//!
//! Appearance depends on the terminal's interpretation of that palette slot.

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
///
/// Emission preserves the palette index rather than converting it to another color model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
// NOTE(invariant): None.
pub struct Ansi8Bit(u8);

impl Ansi8Bit {
    /// Numeric palette slot preserved by indexed-color emission.
    #[inline]
    pub const fn unwrap(self) -> u8 {
        let Self(inner) = self;

        inner
    }

    /// Select a slot in the terminal's 256-color palette.
    /// Every byte value is a valid index.
    #[inline]
    pub const fn raw(value: u8) -> Self {
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
