//! SGR codes and sets of active text attributes.
//!
//! [`TextAttribute`] identifies an individual SGR attribute, while [`Attributes`]
//! collects attributes for emission. This lets styling code combine attributes
//! before writing the corresponding terminal sequences.

use core::fmt;

use aldebaran_print::prelude::{Combine, Print};

use crate::{prelude::AnsiSequence, reset::Resettable};

/// A supported text-presentation setting with activation and selective reset codes.
/// Resetting an attribute clears it without reconstructing earlier terminal state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum TextAttribute {
    /// An attribute that makes the text **bold**.
    Bold = 1,

    /// An attribute that makes the text dim.
    Dim = 2,

    /// An attribute that makes the text italic.
    Italic = 3,

    /// An attribute that makes the text <ins>underlined</ins>.
    Underline = 4,

    /// An attribute that makes the text blink.
    Blink = 5,

    /// An attribute that swaps the foreground and background appearance.
    Inverse = 7,

    /// An attribute that makes the text invisible.
    Invisible = 8,

    /// An attribute that makes the text <del>strikethrough</del>.
    Strikethrough = 9,
}

impl TextAttribute {
    /// The SGR code for resetting active text and color settings.
    pub const RESET_ALL_CODE: u8 = 0;

    /// SGR parameter that activates this setting, without escape-sequence framing.
    #[inline]
    pub const fn active_code(&self) -> u8 {
        *self as u8
    }

    /// SGR parameter that clears this setting, without restoring previous state.
    ///
    /// Bold and dim share SGR 22, which resets both intensity attributes.
    #[inline]
    pub const fn reset_code(&self) -> u8 {
        match self {
            Self::Bold | Self::Dim => 22,
            Self::Italic => 23,
            Self::Underline => 24,
            Self::Blink => 25,
            Self::Inverse => 27,
            Self::Invisible => 28,
            Self::Strikethrough => 29,
        }
    }

    /// Return the attribute for a supported activation code. Other codes yield
    /// [`Option::None`].
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

/// A set of active [`TextAttribute`] values.
///
/// Iteration consumes the set in descending SGR code order. Sequence emission
/// uses a copy, so emitting or resetting the set preserves its stored activations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
// NOTE(invariant): SetBits(Attributes) SubsetOf Discriminants(TextAttribute)
pub struct Attributes(u16);

impl Attributes {
    /// No attribute activations. Emitting or resetting this set writes nothing.
    #[inline]
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Select an isolated activation, with all other attributes absent.
    #[inline]
    pub const fn single(attr: TextAttribute) -> Self {
        Self(1 << attr as u8)
    }
}

impl Attributes {
    /// Toggle whether the supplied [`TextAttribute`] is active.
    #[inline]
    pub const fn toggle(&mut self, attr: TextAttribute) {
        let &mut Self(ref mut target_bitfield) = self;

        *target_bitfield ^= 1 << attr as u8;
    }

    /// Mark the supplied [`TextAttribute`] active.
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

/// Generate attribute query and mutation methods for [`Attributes`].
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
                    #[doc = concat!("Whether `", stringify!($display_name), "` is selected for subsequent attribute emission.")]
                    #[inline]
                    pub const fn [< is_ $display_name >]:concatenate (&self) -> bool {
                        let &Self(target_value) = self;

                        let target_rhs = 1 << $target_variant as u16;

                        target_value & target_rhs != 0
                    }

                    #[doc = concat!("Toggle the status of the `", stringify!($display_name), "` attribute.")]
                    ///
                    /// Returns whether the attribute was active before this call.
                    #[inline]
                    pub const fn [< toggle_ $display_name >]:concatenate (&mut self) -> bool {
                        let &mut Self(target_value) = self;

                        let target_rhs = 1 << $target_variant as u16;

                        let last_state = target_value & target_rhs != 0;

                        let target_update = target_value ^ target_rhs;

                        self.0 = target_update;

                        last_state
                    }

                    #[doc = concat!("Select `", stringify!($display_name), "` for emission while preserving other attributes.")]
                    ///
                    /// Returns whether the attribute was active before this call.
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

/// Emit a [`TextAttribute`] activation or an SGR 0 reset when the option is absent.
///
/// SGR 0 clears all active SGR settings. Use [`crate::reset::Reset`] to emit an
/// attribute's reset code instead.
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

#[cfg(test)]
mod tests {
    use crate::reset::Resettable;

    use super::{Attributes, TextAttribute};

    #[test]
    fn attribute_resets_use_selective_sgr_codes() {
        let cases = [
            (TextAttribute::Bold, 22),
            (TextAttribute::Dim, 22),
            (TextAttribute::Italic, 23),
            (TextAttribute::Underline, 24),
            (TextAttribute::Blink, 25),
            (TextAttribute::Inverse, 27),
            (TextAttribute::Invisible, 28),
            (TextAttribute::Strikethrough, 29),
        ];

        for (attribute, expected) in cases {
            assert_eq!(attribute.reset_code(), expected, "incorrect reset for {attribute:?}");

            let mut output = String::new();

            attribute.reset(&mut output).expect("attribute reset succeeds");
            assert_eq!(output, format!("\x1b[{expected}m"));
        }
    }

    #[test]
    fn bold_attribute_set_resets_intensity() {
        let attributes = Attributes::single(TextAttribute::Bold);
        let mut output = String::new();

        attributes.reset(&mut output).expect("attribute set reset succeeds");

        assert_eq!(output, "\x1b[22m");
    }
}
