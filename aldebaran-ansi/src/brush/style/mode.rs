//! ANSI text attributes and their compact bitfield representation.
//!
//! [`TextAttribute`] maps semantic text modes to activation and reset codes.
//! [`Attributes`] stores several active modes in one bitfield and can emit or
//! reset each selected attribute without allocating a collection.

use core::fmt;

use aldebaran_print::prelude::{Combine, Print};

use crate::{prelude::AnsiSequence, reset::Resettable};

/// The stylistic settings that can be applied to text.
///
/// To retrieve the control indice assigned to each respective variant,
/// simply cast the enum to an [`u8`].
///
/// ## Remarks
///
/// Be aware that not all terminals may support or show a noticeable difference
/// when using these attributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum TextAttribute {
    /// An attribute that makes the text **bold**.
    Bold = 1,

    /// An attribute that makes the text dim.
    Dim = 2,

    /// An attribute that makes the text italic*.
    Italic = 3,

    /// An attribute that makes the text <ins>underlined</ins>.
    Underline = 4,

    /// An attribute that makes the text blink.
    ///
    /// May not work on all terminals or have a discernible effect.
    Blink = 5,

    /// An attribute that inverses the surface identity of text's foreground
    /// and background colors.
    Inverse = 7,

    /// An attribute that makes the text invisible.
    Invisible = 8,

    /// An attribute that makes the text <del>strikethrough</del>.
    Strikethrough = 9,
}

impl TextAttribute {
    /// The ANSI code that resets all the stylistic settings to their default
    /// value.
    pub const RESET_ALL_CODE: u8 = 0;

    /// The offset between a regular activation sequence and a reset sequence.
    const RESET_CODE_OFFSET: u8 = 20;

    /// Retrieve the ANSI code to activate the current attribute.
    #[inline]
    pub const fn active_code(&self) -> u8 {
        *self as u8
    }

    /// Retrieve the ANSI code to reset the current attribute.
    #[inline]
    pub const fn reset_code(&self) -> u8 {
        *self as u8 + Self::RESET_CODE_OFFSET
    }

    /// Determine the corresponding [`TextAttribute`] variant for the given
    /// code.
    #[inline]
    pub const fn from_code(code: u8) -> Option<Self> {
        match code {
            1 => Some(Self::Bold),
            2 => Some(Self::Dim),
            3 => Some(Self::Italic),
            4 => Some(Self::Underline),
            5 => Some(Self::Blink),
            7 => Some(Self::Inverse),
            8 => Some(Self::Invisible),
            9 => Some(Self::Strikethrough),
            _ => None,
        }
    }
}

/// A bitfield that consists of the text attributes that can be applied to text,
/// as seen in [`TextAttribute`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Attributes(u16);

impl Attributes {
    /// Create a new empty set of attributes.
    #[inline]
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Create a new set of attributes from a single [`TextAttribute`].
    #[inline]
    pub const fn single(attr: TextAttribute) -> Self {
        Self(1 << attr as u8)
    }
}

impl Attributes {
    /// Toggle the existence of some [`TextAttribute`] in the set.
    #[inline]
    pub const fn toggle(&mut self, attr: TextAttribute) {
        let &mut Self(ref mut target_bitfield) = self;

        *target_bitfield ^= 1 << attr as u8;
    }

    /// Force the existence of some [`TextAttribute`] in the set.
    #[inline]
    pub const fn force(&mut self, attr: TextAttribute) {
        let &mut Self(ref mut target_bitfield) = self;

        *target_bitfield |= 1 << attr as u8;
    }
}

impl Iterator for Attributes {
    type Item = TextAttribute;

    fn next(&mut self) -> Option<Self::Item> {
        let &mut Self(ref mut target_value) = self;

        let msb = target_value.checked_ilog2()?;

        *target_value &= !(1 << msb);

        TextAttribute::from_code(msb as u8)
    }
}

/// Implement the myriad of `is_*`, 'toggle_*', ... methods for the `Attributes`
/// bitfield.
macro_rules! method_impl {
    () => {};
    (
        $target_struct:ident for {
            $($target_variant:path as $display_name:ident),+
        }
    ) => {
        tokel::stream! {
            impl $target_struct {
                $(
                    #[doc = concat!("Determine whether the text attributes contain the `", stringify!($display_name), "` attribute.")]
                    #[inline]
                    pub const fn [< is_ $display_name >]:concatenate (&self) -> bool {
                        let &Self(target_value) = self;

                        let target_rhs = 1 << $target_variant as u16;

                        target_value & target_rhs != 0
                    }

                    #[doc = concat!("Toggle the status of the `", stringify!($display_name), "` attribute.")]
                    ///
                    /// Returns the last state of the attribute.
                    #[inline]
                    pub const fn [< toggle_ $display_name >]:concatenate (&mut self) -> bool {
                        let &mut Self(target_value) = self;

                        let target_rhs = 1 << $target_variant as u16;

                        let last_state = target_value & target_rhs != 0;

                        let target_update = target_value ^ target_rhs;

                        self.0 = target_update;

                        last_state
                    }

                    #[doc = concat!("Set the status of the `", stringify!($display_name), "` attribute.")]
                    ///
                    /// Returns the last state of the attribute.
                    #[inline]
                    pub const fn $display_name (&mut self) -> bool {
                        let &mut Self(target_value) = self;

                        let target_rhs = 1 << $target_variant as u16;

                        let last_state = target_value & target_rhs != 0;

                        let target_update = target_value | target_rhs;

                        self.0 = target_update;

                        last_state
                    }
                )+
            }
        }
    };
}

method_impl!(
    Attributes for {
        TextAttribute::Bold as bold,
        TextAttribute::Dim as dim,
        TextAttribute::Italic as italic,
        TextAttribute::Underline as underline,
        TextAttribute::Blink as blink,
        TextAttribute::Inverse as inverse,
        TextAttribute::Invisible as invisible,
        TextAttribute::Strikethrough as strikethrough
    }
);

impl AnsiSequence<()> for TextAttribute {
    #[inline]
    fn emit_with_input<W>(&self, writer: &mut W, _: ()) -> fmt::Result
    where
        W: fmt::Write,
    {
        let target_code = self.active_code();

        '\x1B'.sequence('[').sequence(target_code).sequence('m').print(writer)
    }
}

impl Resettable<()> for TextAttribute {
    #[inline]
    fn reset<W>(&self, writer: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        let target_code = self.reset_code();

        '\x1B'.sequence('[').sequence(target_code).sequence('m').print(writer)
    }
}

/// An [`AnsiSequence`] implementation for `Option<TextAttribute>`.
///
/// In most cases, [`TextAttribute`] is wrapped inside an [`Option<T>`], and
/// for ease-of-use, if [`None`] is encountered, an all-resetting sequence is
/// sent through instead.
///
/// For a way to reset a speficic [`TextAttribute`], please see [`crate::reset::Reset`].
impl<Input> AnsiSequence<Input> for Option<TextAttribute>
where
    TextAttribute: AnsiSequence<Input>,
{
    #[inline]
    fn emit_with_input<W>(&self, writer: &mut W, input: Input) -> fmt::Result
    where
        W: fmt::Write,
    {
        match self {
            Some(target_value) => target_value.emit_with_input(writer, input),
            None => '\x1B'
                .sequence('[')
                .sequence(TextAttribute::RESET_ALL_CODE)
                .sequence('m')
                .print(writer),
        }
    }
}

impl AnsiSequence<()> for Attributes {
    #[inline]
    fn emit_with_input<W>(&self, writer: &mut W, input: ()) -> fmt::Result
    where
        W: fmt::Write,
    {
        let target_value = *self;

        for ref attr in target_value {
            AnsiSequence::<()>::emit_with_input(attr, writer, input)?;
        }

        Ok(())
    }
}

impl Resettable<()> for Attributes {
    #[inline]
    fn reset<W>(&self, writer: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        let target_value = *self;

        for ref attr in target_value {
            Resettable::<()>::reset(attr, writer)?;
        }

        Ok(())
    }
}
