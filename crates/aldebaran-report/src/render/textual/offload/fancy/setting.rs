//! Configure the presentation of [`Fancy`] diagnostics.
//!
//! Start from a [`Fancyness`] preset or customize [`FancySettings`] directly.
//!
//! Styling and glyph selection are independent, so Unicode output does not
//! require ANSI styling.
//!
//! Gutter and tab spacing follow these settings. Source wrapping, however,
//! follows the viewport supplied for rendering.
//!
//! [`Fancy`]: super::Fancy

use core::num::NonZero;

use crate::render::textual::offload::fancy::{
    charset::Charset,
    layout::LayoutSettings,
    theme::{DefaultTheme, Theme},
};

/// Styling and glyph presets for [`Fancy`].
///
/// Use [`FancySettings::level`] to apply a preset, then customize individual
/// settings as needed.
///
/// [`Fancy`]: super::Fancy
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
pub enum Fancyness {
    /// ANSI styling with Unicode framing and directional cues.
    Outblown,

    /// ANSI styling with ASCII framing and indicators.
    Medium,

    /// Plain text with ASCII framing and indicators, the default preset.
    #[default]
    Meek,
}

impl Fancyness {
    /// Whether this preset enables ANSI colors and text attributes.
    #[inline]
    pub const fn colored(&self) -> bool {
        match self {
            Self::Outblown | Self::Medium => true,
            Self::Meek => false,
        }
    }

    /// Whether this preset uses Unicode glyphs rather than ASCII.
    #[inline]
    pub const fn unicode(&self) -> bool {
        match self {
            Self::Outblown => true,
            Self::Medium | Self::Meek => false,
        }
    }
}

/// Controls ANSI colors and text attributes without changing the selected glyphs.
///
/// Styling is disabled by default, so a Unicode charset can be used with plain
/// output.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Colored {
    /// Enable theme colors and text attributes.
    Yes,
    /// Render without ANSI styling.
    No,
}

impl Colored {
    /// Plain output, also used by [`Default`].
    #[inline]
    pub const fn standard() -> Self {
        Self::No
    }
}

impl Colored {
    /// Enable ANSI styling when `value` is `true`.
    #[inline]
    pub const fn bool(value: bool) -> Self {
        if value { Self::Yes } else { Self::No }
    }

    /// Whether ANSI colors and attributes are enabled.
    #[inline]
    pub const fn colored(&self) -> bool {
        match self {
            Self::Yes => true,
            Self::No => false,
        }
    }
}

impl Default for Colored {
    #[inline]
    fn default() -> Self {
        Self::standard()
    }
}

/// Presentation settings for [`Fancy`]'s heading and source excerpt.
///
/// Styling, glyphs, gutter spacing, and tab stops can be customized independently.
///
/// The render's viewport controls source wrapping. It does not change the chosen
/// gutter layout.
///
/// [`Fancy`]: super::Fancy
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
// NOTE(invariant): None.
pub struct FancySettings<T = DefaultTheme>
where
    T: Theme,
{
    /// Whether the theme's colors and attributes appear in output.
    colored: Colored,

    /// Gutter spacing, with `None` choosing a layout from the viewport.
    layout: Option<LayoutSettings>,

    /// Glyphs for source framing and annotation markers.
    charset: Charset,

    /// Tab interval in terminal cells, measured after the gutter.
    tab_width: NonZero<u16>,

    /// Appearance of headings, line numbers, and annotation markers.
    theme: T,
}

impl FancySettings {
    /// Plain ASCII output with [`DefaultTheme`] and [`LayoutSettings::standard`].
    #[inline]
    pub const fn standard() -> Self {
        let colored = Colored::standard();
        let layout = Some(LayoutSettings::standard());
        let charset = Charset::standard();
        let tab_width = const { NonZero::new(4).expect("four is nonzero") };
        let theme = DefaultTheme;

        Self {
            colored,
            layout,
            charset,
            tab_width,
            theme,
        }
    }
}

impl<T> FancySettings<T>
where
    T: Theme,
{
    /// Apply a preset with a default theme and [`LayoutSettings::standard`].
    #[inline]
    pub fn level(fancyness: Fancyness) -> Self
    where
        T: Default,
    {
        let colored = Colored::bool(fancyness.colored());
        let layout = Some(LayoutSettings::standard());
        let charset = Charset::fancyness(fancyness);
        let tab_width = NonZero::new(4).expect("four is nonzero");
        let theme = T::default();

        Self {
            colored,
            layout,
            charset,
            tab_width,
            theme,
        }
    }

    /// Use an explicit layout and charset with `T`'s default theme.
    #[inline]
    pub fn tuple((colored, layout, charset): (Colored, LayoutSettings, Charset)) -> Self
    where
        T: Default,
    {
        let layout = Some(layout);
        let theme = T::default();
        let tab_width = NonZero::new(4).expect("four is nonzero");

        Self {
            colored,
            layout,
            charset,
            tab_width,
            theme,
        }
    }
    /// Use an explicit theme with the chosen styling, layout, and glyphs.
    #[inline]
    pub const fn raw(colored: Colored, layout: LayoutSettings, charset: Charset, theme: T) -> Self {
        let layout = Some(layout);
        let tab_width = NonZero::new(4).expect("four is nonzero");

        Self {
            colored,
            layout,
            charset,
            tab_width,
            theme,
        }
    }

    /// Whether ANSI colors and attributes are enabled.
    #[inline]
    pub const fn colored(&self) -> Colored {
        let &Self { colored, .. } = self;

        colored
    }

    /// Gutter layout, or `None` for spacing chosen from the viewport.
    #[inline]
    pub const fn layout(&self) -> Option<LayoutSettings> {
        let &Self { layout, .. } = self;

        layout
    }

    /// Glyphs used for source framing and annotation markers.
    #[inline]
    pub const fn charset(&self) -> Charset {
        let &Self { charset, .. } = self;

        charset
    }

    /// Tab stops restart after the gutter on each wrapped source row, with a
    /// default interval of four terminal cells.
    #[inline]
    pub const fn tab_width(&self) -> NonZero<u16> {
        let &Self { tab_width, .. } = self;

        tab_width
    }

    /// Styles for the heading, line numbers, and annotation markers.
    #[inline]
    pub const fn theme(&self) -> &T {
        let &Self { ref theme, .. } = self;

        theme
    }

    /// Borrow the ANSI styling choice.
    #[inline]
    pub const fn colored_ref(&self) -> &Colored {
        let &Self { ref colored, .. } = self;

        colored
    }

    /// Borrow the gutter override, if present.
    #[inline]
    pub const fn layout_ref(&self) -> Option<&LayoutSettings> {
        let &Self { ref layout, .. } = self;

        layout.as_ref()
    }

    /// Borrow the framing and marker glyphs.
    #[inline]
    pub const fn charset_ref(&self) -> &Charset {
        let &Self { ref charset, .. } = self;

        charset
    }

    /// Borrow the styles for diagnostic content.
    #[inline]
    pub const fn theme_ref(&self) -> &T {
        let &Self { ref theme, .. } = self;

        theme
    }

    /// Change ANSI styling independently of glyph selection.
    #[inline]
    pub const fn colored_mut(&mut self) -> &mut Colored {
        let &mut Self { ref mut colored, .. } = self;

        colored
    }

    /// Change the gutter override, returning `None` when spacing follows the viewport.
    #[inline]
    pub const fn layout_mut(&mut self) -> Option<&mut LayoutSettings> {
        let &mut Self { ref mut layout, .. } = self;

        layout.as_mut()
    }

    /// Change glyphs independently of ANSI styling.
    #[inline]
    pub const fn charset_mut(&mut self) -> &mut Charset {
        let &mut Self { ref mut charset, .. } = self;

        charset
    }

    /// Change the tab interval in terminal cells.
    #[inline]
    pub const fn tab_width_mut(&mut self) -> &mut NonZero<u16> {
        let &mut Self { ref mut tab_width, .. } = self;

        tab_width
    }
}

impl Default for FancySettings {
    #[inline]
    fn default() -> Self {
        Self::standard()
    }
}

#[cfg(test)]
mod tests {
    use super::{Charset, Colored, DefaultTheme, FancySettings, Fancyness, LayoutSettings, NonZero, Theme};
    use crate::{render::textual::offload::fancy::theme::HeaderTheme, report::severity::Metadata};

    #[derive(Clone, Copy, Default)]
    struct CustomTheme;

    impl Theme for CustomTheme {
        #[inline]
        fn metadata(&self, metadata: &Metadata) -> HeaderTheme {
            DefaultTheme.metadata(metadata)
        }
    }

    #[test]
    fn constructors_default_to_four_cells() {
        let standard = const { FancySettings::standard() };
        let default = FancySettings::default();
        let layout = LayoutSettings::standard();
        let charset = Charset::standard();
        let tuple = FancySettings::<DefaultTheme>::tuple((Colored::No, layout, charset));
        let raw = const {
            let layout = LayoutSettings::standard();
            let charset = Charset::standard();

            FancySettings::raw(Colored::No, layout, charset, DefaultTheme)
        };

        for settings in [standard, default, tuple, raw] {
            assert_eq!(settings.tab_width().get(), 4);
        }

        for level in [Fancyness::Meek, Fancyness::Medium, Fancyness::Outblown] {
            let settings = FancySettings::<DefaultTheme>::level(level);

            assert_eq!(settings.tab_width().get(), 4);
        }
    }

    #[test]
    fn custom_theme_constructors_default_to_four_cells() {
        let layout = LayoutSettings::standard();
        let charset = Charset::standard();
        let tuple = FancySettings::<CustomTheme>::tuple((Colored::Yes, layout, charset));
        let raw = FancySettings::raw(Colored::Yes, layout, charset, CustomTheme);

        for settings in [tuple, raw] {
            assert_eq!(settings.tab_width().get(), 4);
        }

        for level in [Fancyness::Meek, Fancyness::Medium, Fancyness::Outblown] {
            let settings = FancySettings::<CustomTheme>::level(level);

            assert_eq!(settings.tab_width().get(), 4);
        }
    }

    #[test]
    fn tab_width_supports_nonzero_customization() {
        let width = const {
            let mut settings = FancySettings::standard();

            *settings.tab_width_mut() = NonZero::new(8).expect("eight is nonzero");

            settings.tab_width()
        };

        assert_eq!(width.get(), 8);

        let mut settings = FancySettings::<CustomTheme>::level(Fancyness::Medium);

        for width in [NonZero::<u16>::MIN, NonZero::<u16>::MAX] {
            *settings.tab_width_mut() = width;

            assert_eq!(settings.tab_width(), width);
        }
    }
}
