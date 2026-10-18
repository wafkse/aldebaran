//! Terminal styles combining color overrides and text attributes.
//!
//! [`Style`] emits [`Attributes`] and optional foreground and background
//! colors. Its [`Stylus`] implementation brackets a writer closure with
//! activation and a global reset, without restoring earlier terminal settings.

mod mode;

use core::fmt;

use aldebaran_print::prelude::{Combine, Print};

use aldebaran_print::style::Stylus;

use crate::{
    prelude::{AnsiSequence, Color, Surface},
    reset::{Reset, Resettable},
};

pub use mode::{Attributes, TextAttribute};

/// Optional [`Color`] overrides and active [`Attributes`] for formatted output.
///
/// The [`Stylus`] implementation applies the configured SGR state around a
/// closure's output, leaving a surface unchanged during setup when its color is
/// absent. After the closure returns, SGR 0 clears active settings, so wrapping
/// output does not restore the terminal's previous state.
///
/// If setup fails, the closure does not run. Otherwise, its result is returned
/// only after reset succeeds, which means a reset failure takes precedence over
/// the closure's result. A panic bypasses this reset path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
// NOTE(invariant): None.
pub struct Style {
    /// The configured foreground override, if present.
    foreground: Option<Color>,

    /// The configured background override, if present.
    background: Option<Color>,

    /// Text attributes selected for emission.
    attribute: Attributes,
}

impl Style {
    /// No activation settings. Used as a [`Stylus`], this still emits a global reset after output.
    #[inline]
    pub const fn empty() -> Self {
        Self {
            foreground: None,
            background: None,
            attribute: Attributes::empty(),
        }
    }

    /// Glyph-color override. An absent override leaves the foreground unchanged during activation.
    #[inline]
    pub const fn foreground(&self) -> Option<Color> {
        let &Self { foreground, .. } = self;

        foreground
    }

    /// Adjust an existing glyph-color override.
    /// An absent override cannot be installed through this view.
    #[inline]
    pub const fn foreground_mut(&mut self) -> Option<&mut Color> {
        let &mut Self { ref mut foreground, .. } = self;

        foreground.as_mut()
    }

    /// Background override. An absent override leaves the background unchanged during activation.
    #[inline]
    pub const fn background(&self) -> Option<Color> {
        let &Self { background, .. } = self;

        background
    }

    /// Adjust an existing background override.
    /// An absent override cannot be installed through this view.
    #[inline]
    pub const fn background_mut(&mut self) -> Option<&mut Color> {
        let &mut Self { ref mut background, .. } = self;

        background.as_mut()
    }

    /// Attribute activations included in this style's emitted settings.
    #[inline]
    pub const fn attribute(&self) -> Attributes {
        let &Self { attribute, .. } = self;

        attribute
    }

    /// Adjust the attributes emitted by this style on subsequent activation.
    #[inline]
    pub const fn attribute_mut(&mut self) -> &mut Attributes {
        let &mut Self { ref mut attribute, .. } = self;

        attribute
    }

    /// Replace the attribute set and return its previous value.
    #[inline]
    pub const fn attribute_replace(&mut self, value: Attributes) -> Attributes {
        let &mut Self { ref mut attribute, .. } = self;

        let last_value = *attribute;

        *attribute = value;

        last_value
    }

    /// Set the foreground override and return the previous value, if present.
    #[inline]
    pub const fn foreground_replace(&mut self, value: Color) -> Option<Color> {
        let &mut Self { ref mut foreground, .. } = self;

        foreground.replace(value)
    }

    /// Set the background override and return the previous value, if present.
    #[inline]
    pub const fn background_replace(&mut self, value: Color) -> Option<Color> {
        let &mut Self { ref mut background, .. } = self;

        background.replace(value)
    }
}

impl<W> Stylus<W> for Style
where
    W: fmt::Write,
{
    type Error = fmt::Error;

    fn style<F, O, E>(&self, writer: &mut W, closure: F) -> Result<Result<O, E>, Self::Error>
    where
        F: FnOnce(&mut W) -> Result<O, E>,
    {
        self.emit(writer)?;

        let target_result = closure(writer);

        Reset::this(self).resettable().reset(writer)?;

        Ok(target_result)
    }
}

impl AnsiSequence<()> for Style {
    fn emit_with_input<W>(&self, writer: &mut W, _: ()) -> fmt::Result
    where
        W: fmt::Write,
    {
        let Self {
            foreground,
            background,
            attribute,
        } = self;

        match (foreground, background, attribute) {
            // Palette parameters and attributes share a single SGR sequence.
            (&Some(Color::Bits3(foreground_color)), &Some(Color::Bits3(background_color)), &target_attributes) => {
                let foreground_code = foreground_color.foreground();
                let background_code = background_color.background();

                '\x1B'.sequence('[').print(writer)?;

                for attr in target_attributes {
                    attr.active_code().sequence(';').print(writer)?;
                }

                foreground_code.sequence(';').sequence(background_code).sequence('m').print(writer)
            }
            (foreground_color, background_color, attributes) => {
                attributes.emit(writer)?;

                if let Some(foreground_color) = foreground_color {
                    foreground_color.emit_with_input(writer, Surface::Foreground)?;
                }

                if let Some(background_color) = background_color {
                    background_color.emit_with_input(writer, Surface::Background)?;
                }

                Ok(())
            }
        }
    }
}

impl<T> Resettable<T> for Style
where
    Self: AnsiSequence<T>,
{
    #[inline]
    fn reset<W>(&self, writer: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        '\x1B'.sequence('[').sequence(0).sequence('m').print(writer)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        brush::{
            color::{Color, bits3::Ansi3Bit, rgb::AnsiRgb},
            style::{Attributes, TextAttribute},
        },
        prelude::AnsiSequence,
    };

    use aldebaran_print::prelude::Print;

    use crate::prelude::Paintable;

    use super::Style;

    #[test]
    fn painted_attributes_survive_color_until_final_reset() {
        let mut output = String::new();

        "subject".red().bold().underline().print(&mut output).expect("paint succeeds");

        assert_eq!(output, "\x1b[4m\x1b[1m\x1b[31msubject\x1b[0m");
    }

    #[test]
    fn colors_preserve_active_attributes() {
        let cases = [
            (Some(Color::Bits3(Ansi3Bit::Red)), None, "\x1b[4m\x1b[1m\x1b[31m"),
            (None, Some(Color::Bits3(Ansi3Bit::Blue)), "\x1b[4m\x1b[1m\x1b[44m"),
            (
                Some(Color::Bits3(Ansi3Bit::Red)),
                Some(Color::Bits3(Ansi3Bit::Blue)),
                "\x1b[4;1;31;44m",
            ),
            (
                Some(Color::Rgb(AnsiRgb::RED)),
                Some(Color::Bits3(Ansi3Bit::Blue)),
                "\x1b[4m\x1b[1m\x1b[38;2;255;0;0m\x1b[44m",
            ),
            (
                Some(Color::Bits3(Ansi3Bit::Red)),
                Some(Color::Rgb(AnsiRgb::BLUE)),
                "\x1b[4m\x1b[1m\x1b[31m\x1b[48;2;0;0;255m",
            ),
        ];

        for (foreground, background, expected) in cases {
            let mut attribute = Attributes::single(TextAttribute::Bold);

            attribute.force(TextAttribute::Underline);

            let style = Style {
                foreground,
                background,
                attribute,
            };
            let mut output = String::new();

            style.emit(&mut output).expect("style emission succeeds");

            assert_eq!(output, expected, "color activation must not reset attributes");
        }
    }
}
