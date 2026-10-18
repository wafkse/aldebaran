//! Component capabilities for one-dimensional source sequences.
//!
//! [`Component`] defines the width used to advance source coordinates.
//! Additional ASCII and digit capabilities let text algorithms classify [`u8`]
//! and [`char`] without forcing every source component into one representation.

use core::{mem, num::NonZero};

use aldebaran_print::visualize::Visualize;

/// A source value with a nonzero extent in its source's coordinate units.
pub trait Component: Copy + PartialEq + Eq + PartialOrd + Ord + Visualize + 'static {
    /// Return this component's width in source-coordinate units.
    ///
    /// The width must use the same unit as the source's
    /// [`footprint`](crate::source::Source::footprint), so traversing components
    /// advances coordinates consistently with source spans. For string [`char`]
    /// components this means UTF-8 byte widths, whereas byte slices advance by
    /// one coordinate unit per byte.
    fn size(&self) -> NonZero<usize>;
}

/// Lossless ASCII classification shared by byte and character sources.
/// Construction accepts only `0x00..=0x7f`, so conversion to a character preserves the value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
// NOTE(invariant): Value In 0x00..=0x7f
pub struct AsciiByte(u8);

impl AsciiByte {
    /// Return an `AsciiByte` when `value` is in `0x00..=0x7f`.
    #[inline]
    #[must_use]
    pub const fn new(value: u8) -> Option<Self> {
        if value.is_ascii() { Some(Self(value)) } else { None }
    }

    /// ASCII code in `0x00..=0x7f`, suitable for byte-oriented classification.
    #[inline]
    #[must_use]
    pub const fn get(self) -> u8 {
        let Self(value) = self;

        value
    }

    /// Unicode scalar with the same ASCII code, including control characters.
    #[inline]
    #[must_use]
    pub const fn as_char(self) -> char {
        // ASCII codes have the same numeric values in Unicode.
        self.get() as char
    }
}

/// ASCII classification without changing a source's component representation.
pub trait AsciiComponent: Component {
    /// Return its ASCII byte when the component represents a value in `0x00..=0x7f`.
    fn ascii(self) -> Option<AsciiByte>;
}

/// Source component capability for positional-radix digits.
pub trait DigitComponent: Component {
    /// Return the digit value when this component is valid in `radix`.
    ///
    /// The supported radix range is 2 through 36, following
    /// [`char::to_digit`].
    fn digit(self, radix: u32) -> Option<u32>;
}

/// Shared text-processing bound for ASCII and radix-digit recognition.
/// Both character and byte sources satisfy it without converting their stored components.
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

/// Implement [`Component`] for types whose built-in operation returns their
/// width in source-coordinate units.
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
