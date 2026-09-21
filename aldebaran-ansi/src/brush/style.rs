//! Composite ANSI styles for painted output.
//!
//! [`Style`] combines optional foreground and background colors with a set of
//! text attributes. Emission coalesces compatible settings where possible and
//! the [`Stylus`] implementation resets terminal state after the wrapped print.

mod mode;

use core::fmt;

use aldebaran_print::prelude::{Combine, Print};

use aldebaran_style::prelude::Stylus;

use crate::{
    prelude::{AnsiSequence, Color, Surface},
    reset::{Reset, Resettable},
};

pub use mode::{Attributes, TextAttribute};

/// A style that can be applied to a painted type.
///
/// The stylistic settings that can be applied are:
/// - `foreground` color
/// - `background` color
/// - `bold` setting
/// - `underline` setting
/// - `strike` setting (strike-through text)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Style {
    foreground: Option<Color>,
    background: Option<Color>,
    attribute: Attributes,
}

/// Main implementation block for [`Style`].
///
/// Methods for this type follow a determinate trend:
///
/// - Immutable (copied) accessors are named after the field they access.
/// - Mutable accessors are named after the field they access, suffixed with `_mut`.
/// - Replace methods are named after the field they replace, suffixed with `_replace`.
impl Style {
    /// Create a new, empty, [`Style`].
    ///
    /// This style has no foreground, background, or bold settings (`defaults to
    /// false`)
    #[inline]
    pub const fn empty() -> Self {
        Self {
            foreground: None,
            background: None,
            attribute: Attributes::empty(),
        }
    }

    /// Retrieve the underlying `foreground` color of this [`Style`].
    ///
    /// To retrieve a mutable reference to the foreground color, use the
    /// [`Style::foreground_mut`] method instead.
    #[inline]
    pub const fn foreground(&self) -> Option<Color> {
        let &Self { foreground, .. } = self;

        foreground
    }

    /// Retrieve a mutable reference to the underlying `foreground` color of
    /// this [`Style`].
    ///
    /// If you wish to only retrieve the associated setting, make use of the
    /// [`Style::foreground`] method instead.
    #[inline]
    pub const fn foreground_mut(&mut self) -> Option<&mut Color> {
        let &mut Self { ref mut foreground, .. } = self;

        foreground.as_mut()
    }

    /// Retrieve the underlying `background` color of this [`Style`].
    ///
    /// To retrieve a mutable reference to the background color, use the
    /// [`Style::background_mut`] method instead.
    #[inline]
    pub const fn background(&self) -> Option<Color> {
        let &Self { background, .. } = self;

        background
    }

    /// Retrieve a mutable reference to the underlying `background` color of
    /// this [`Style`].
    ///
    /// If you wish to only retrieve the associated setting, make use of the
    /// [`Style::background`] method instead.
    #[inline]
    pub const fn background_mut(&mut self) -> Option<&mut Color> {
        let &mut Self { ref mut background, .. } = self;

        background.as_mut()
    }

    /// Retrieve the underlying [`TextAttribute`] (if existing) of this
    /// [`Style`].
    #[inline]
    pub const fn attribute(&self) -> Attributes {
        let &Self { attribute, .. } = self;

        attribute
    }

    /// Retrieve a mutable reference to the underlying [`TextAttribute`] (if
    /// existing) of this [`Style`].
    #[inline]
    pub const fn attribute_mut(&mut self) -> &mut Attributes {
        let &mut Self { ref mut attribute, .. } = self;

        attribute
    }

    /// Replace (alike to [`Option::replace`]) the `attribute` setting of this
    /// [`Style`] with the specified one.
    ///
    /// For a mutable reference to the `attribute` setting, use the
    /// [`Style::attribute_mut`] method instead.
    #[inline]
    pub const fn attribute_replace(&mut self, value: Attributes) -> Attributes {
        let &mut Self { ref mut attribute, .. } = self;

        let last_value = *attribute;

        *attribute = value;

        last_value
    }

    /// Replace (alike to [`Option::replace`]) the `foreground` color of this
    /// [`Style`] with the specified one.
    ///
    /// For a mutable reference to the `foreground` color, use the
    /// [`Style::foreground_mut`] method instead.
    #[inline]
    pub const fn foreground_replace(&mut self, value: Color) -> Option<Color> {
        let &mut Self { ref mut foreground, .. } = self;

        foreground.replace(value)
    }

    /// Replace (alike to [`Option::replace`]) the `background` color of this
    /// [`Style`] with the specified one.
    ///
    /// For a mutable reference to the `background` color, use the
    /// [`Style::background_mut`] method instead.
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
            // If all the settings are present, and the foreground and
            // background are both 3-bit colors.
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
