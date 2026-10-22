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

/// A terminal palette slot covering all indices from 0 through 255.
/// Emission preserves the index rather than approximating it with another color model.
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
