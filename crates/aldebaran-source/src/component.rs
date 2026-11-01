//! Component capabilities for one-dimensional source sequences.
//!
//! See [`Component`], [`AsciiComponent`], and [`DigitComponent`] for further information.

use core::{mem, num::NonZero};

use aldebaran_print::visualize::Visualize;

/// A sequence component.
///
/// This trait exposes common functionality to all types that can be used as a
/// component in a sequence.
///
/// Albeit defined generically, this trait is part of a larger scheme: see
/// [`Source`](crate::source::Source).
pub trait Component: Copy + PartialEq + Eq + PartialOrd + Ord + Visualize + 'static {
    /// The size of the component inside its respective one-dimensional plane.
    ///
    /// For detailed documentation on source planes, refer to
    /// [`Source`](crate::source::Source). An arbitrary component must have a size
    /// greater than zero, and its width uses the same unit as the source
    /// footprint. For string [`char`] components this means UTF-8 byte widths,
    /// whereas byte slices advance by one coordinate unit per byte.
    fn size(&self) -> NonZero<usize>;
}

/// One source component proven to be an ASCII byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
// NOTE(invariant): Value In 0x00..=0x7f
pub struct AsciiByte(u8);

impl AsciiByte {
    /// Validate one byte as ASCII.
    ///
    /// Values outside `0x00..=0x7f` return `None`.
    #[inline]
    #[must_use]
    pub const fn new(value: u8) -> Option<Self> {
        if value.is_ascii() { Some(Self(value)) } else { None }
    }

    /// Retrieve the underlying ASCII byte.
    #[inline]
    #[must_use]
    pub const fn get(self) -> u8 {
        let Self(value) = self;

        value
    }

    /// Retrieve the corresponding Unicode scalar value.
    #[inline]
    #[must_use]
    pub const fn as_char(self) -> char {
        // ASCII codes have the same numeric values in Unicode.
        self.get() as char
    }
}

/// Source component capability for lossless ASCII classification.
pub trait AsciiComponent: Component {
    /// Classify this component as ASCII when possible.
    fn ascii(self) -> Option<AsciiByte>;
}

/// Source component capability for positional-radix digits.
pub trait DigitComponent: Component {
    /// Interpret this component as one digit in the requested radix.
    ///
    /// The supported radix range is 2 through 36, following
    /// [`char::to_digit`].
    fn digit(self, radix: u32) -> Option<u32>;
}

/// Source component supporting ordinary textual classification.
///
/// Both character and byte sources satisfy it without converting their stored
/// components.
pub trait TextComponent: Component + AsciiComponent + DigitComponent {}

impl<C> TextComponent for C where C: Component + AsciiComponent + DigitComponent {}

impl AsciiComponent for char {
    #[inline]
    fn ascii(self) -> Option<AsciiByte> {
        let value = u8::try_from(self).ok();

        value.and_then(AsciiByte::new)
    }
}

impl DigitComponent for char {
    #[inline]
    fn digit(self, radix: u32) -> Option<u32> {
        self.to_digit(radix)
    }
}

impl AsciiComponent for u8 {
    #[inline]
    fn ascii(self) -> Option<AsciiByte> {
        AsciiByte::new(self)
    }
}

impl DigitComponent for u8 {
    #[inline]
    fn digit(self, radix: u32) -> Option<u32> {
        (self as char).to_digit(radix)
    }
}

/// A macro for automatic implementation of the [`Component`] trait for types
/// that have a built-in way to determine their size.
macro_rules! lengthed {
    () => {};
    (
        impl {
            $(
                $(#[$type_meta:meta])*
                $target_type:ident use $target_method:path;
            ),* $(,)?
        }
    ) => {
        $(
            impl Component for $target_type {
                #[inline(always)]
                fn size(&self) -> NonZero<usize> {
                    let target_value = *self;

                    NonZero::new($target_method(target_value)).expect("cannot be zero")
                }
            }
        )*
    };
    (
        ref impl {
            $(
                $(#[$type_meta:meta])*
                $target_type:ident use $target_method:path;
            ),* $(,)?
        }
    ) => {
        $(
            impl Component for $target_type {
                #[inline(always)]
                fn size(&self) -> NonZero<usize>  {
                    let target_value = self;

                    NonZero::new($target_method(target_value)).expect("cannot be zero")
                }
            }
        )*
    };
}

lengthed!(
    impl {
        char use char::len_utf8;
    }
);

lengthed!(
    ref impl {
        u8 use mem::size_of_val;
    }
);

#[cfg(test)]
mod tests {
    //! Tests module for the [`component`] module.
    //!
    //! [`component`]: super

    use super::{AsciiByte, AsciiComponent as _, DigitComponent as _, TextComponent};

    fn require_text_component<C>(_: C)
    where
        C: TextComponent,
    {
    }

    #[test]
    fn character_and_byte_components_share_text_capabilities() {
        require_text_component('A');
        require_text_component(b'A');
        assert_eq!('A'.ascii(), AsciiByte::new(b'A'));
        assert_eq!(b'A'.ascii(), AsciiByte::new(b'A'));
        assert_eq!('f'.digit(16), Some(15));
        assert_eq!(b'f'.digit(16), Some(15));
        assert_eq!('λ'.ascii(), None);
        assert_eq!(0xff_u8.ascii(), None);
        assert_eq!(AsciiByte::new(0x7f).map(AsciiByte::get), Some(0x7f));
        assert_eq!(AsciiByte::new(0x80), None);
    }
}
