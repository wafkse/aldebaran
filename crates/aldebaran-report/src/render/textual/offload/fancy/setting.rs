//! Presentation settings for the fancy textual renderer.
//!
//! [`Fancyness`] supplies a color and glyph preset. [`FancySettings`] combines
//! those choices with gutter spacing and a [`Theme`], keeping style selection
//! separate from whether the renderer emits ANSI styling. Layout settings govern
//! the gutter, while the viewport supplied with the render governs source wrapping.
//!
//! [`Fancy`]: super::Fancy

use crate::render::textual::offload::fancy::{
    charset::Charset,
    layout::LayoutSettings,
    theme::{DefaultTheme, Theme},
};

/// Coordinated ANSI styling and glyph presets for the [`Fancy`] renderer.
///
/// [`FancySettings::level`] applies both choices, while the settings' mutable
/// accessors allow color and glyphs to be customized independently afterward.
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
    /// Whether the preset enables ANSI styling for headings, numbers, and markers.
    #[inline]
    pub const fn colored(&self) -> bool {
        match self {
            Self::Outblown | Self::Medium => true,
            Self::Meek => false,
        }
    }

    /// Whether the preset selects Unicode framing and directional cues.
    #[inline]
    pub const fn unicode(&self) -> bool {
        match self {
            Self::Outblown => true,
            Self::Medium | Self::Meek => false,
        }
    }
}

/// Permission to emit ANSI styling for diagnostic headings, numbers, and markers.
///
/// Disabling styling preserves the selected glyphs and spacing. [`Default`] uses
/// [`Self::No`], so Unicode output does not require terminal color support.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Colored {
    /// Apply theme colors and text attributes to styled diagnostic content.
    Yes,
    /// Emit styled diagnostic content as plain text.
    No,
}

impl Colored {
    /// Disable ANSI styling, as [`Default`] does.
    #[inline]
    pub const fn standard() -> Self {
        Self::No
    }
}

impl Colored {
    /// Treat `true` as permission to emit ANSI styling.
    #[inline]
    pub const fn bool(value: bool) -> Self {
        if value { Self::Yes } else { Self::No }
    }

    /// Whether theme colors and attributes may appear in rendered output.
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

/// Presentation policy shared by [`Fancy`]'s heading and source excerpt.
///
/// The theme selects styles for diagnostic roles and [`Colored`] controls their
/// emission. The charset selects visible markers and framing independently.
/// Public constructors install an explicit gutter layout, so changing the render's
/// viewport affects source wrapping without automatically changing that padding.
///
/// [`Fancy`]: super::Fancy
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
// NOTE(invariant): None.
pub struct FancySettings<T = DefaultTheme>
where
    T: Theme,
{
    /// Permission to emit theme colors and attributes on headings, numbers, and markers.
    colored: Colored,

    /// Gutter override, with `None` selecting spacing from the render's viewport shape.
    layout: Option<LayoutSettings>,

    /// Framing glyphs and marker sequence used by the source excerpt.
    charset: Charset,

    /// Style policy for report metadata, line numbers, and annotation markers.
    theme: T,
}

impl FancySettings {
    /// Use plain ASCII output with [`DefaultTheme`] and fixed standard gutter spacing.
    ///
    /// The gutter has one space before the digits and two before the separator,
    /// regardless of the viewport supplied for rendering.
    #[inline]
    pub const fn standard() -> Self {
        let colored = Colored::standard();
        let layout = Some(LayoutSettings::standard());
        let charset = Charset::standard();
        let theme = DefaultTheme;

        Self {
            colored,
            layout,
            charset,
            theme,
        }
    }
}

impl<T> FancySettings<T>
where
    T: Theme,
{
    /// Apply a color and glyph preset with [`LayoutSettings::standard`] and `T`'s default theme.
    #[inline]
    pub fn level(fancyness: Fancyness) -> Self
    where
        T: Default,
    {
        let colored = Colored::bool(fancyness.colored());
        let layout = Some(LayoutSettings::standard());
        let charset = Charset::fancyness(fancyness);
        let theme = T::default();

        Self {
            colored,
            layout,
            charset,
            theme,
        }
    }

    /// Fix styling permission, gutter spacing, and glyphs with `T`'s default theme.
    #[inline]
    pub fn tuple((colored, layout, charset): (Colored, LayoutSettings, Charset)) -> Self
    where
        T: Default,
    {
        let layout = Some(layout);
        let theme = T::default();

        Self {
            colored,
            layout,
            charset,
            theme,
        }
    }
    /// Supply all presentation policies, with gutter spacing fixed across viewports.
    #[inline]
    pub const fn from_raw_parts(colored: Colored, layout: LayoutSettings, charset: Charset, theme: T) -> Self {
        let layout = Some(layout);

        Self {
            colored,
            layout,
            charset,
            theme,
        }
    }

    /// Permission to apply theme colors and attributes in rendered output.
    #[inline]
    pub const fn colored(&self) -> Colored {
        let &Self { colored, .. } = self;

        colored
    }

    /// Gutter spacing override, or `None` to derive spacing from the viewport shape.
    #[inline]
    pub const fn layout(&self) -> Option<LayoutSettings> {
        let &Self { layout, .. } = self;

        layout
    }

    /// Visible framing and annotation markers, independent of ANSI styling permission.
    #[inline]
    pub const fn charset(&self) -> Charset {
        let &Self { charset, .. } = self;

        charset
    }

    /// Style policy shared by the heading, source line numbers, and annotation markers.
    #[inline]
    pub const fn theme(&self) -> &T {
        let &Self { ref theme, .. } = self;

        theme
    }

    /// Stored permission shared by heading, line-number, and marker styling.
    #[inline]
    pub const fn colored_ref(&self) -> &Colored {
        let &Self { ref colored, .. } = self;

        colored
    }

    /// Configured gutter spacing before any viewport-derived fallback.
    #[inline]
    pub const fn layout_ref(&self) -> Option<&LayoutSettings> {
        let &Self { ref layout, .. } = self;

        layout.as_ref()
    }

    /// Framing and indicator configuration, including the marker sequence's current position.
    #[inline]
    pub const fn charset_ref(&self) -> &Charset {
        let &Self { ref charset, .. } = self;

        charset
    }

    /// Theme consulted for styled content even when ANSI emission is disabled.
    #[inline]
    pub const fn theme_ref(&self) -> &T {
        let &Self { ref theme, .. } = self;

        theme
    }

    /// Enable or disable ANSI styling while retaining the chosen glyphs and layout.
    #[inline]
    pub const fn colored_mut(&mut self) -> &mut Colored {
        let &mut Self { ref mut colored, .. } = self;

        colored
    }

    /// Adjust an existing gutter override without changing whether an override is present.
    #[inline]
    pub const fn layout_mut(&mut self) -> Option<&mut LayoutSettings> {
        let &mut Self { ref mut layout, .. } = self;

        layout.as_mut()
    }

    /// Customize framing and marker glyphs independently of the styling permission.
    #[inline]
    pub const fn charset_mut(&mut self) -> &mut Charset {
        let &mut Self { ref mut charset, .. } = self;

        charset
    }
}

impl Default for FancySettings {
    #[inline]
    fn default() -> Self {
        Self::standard()
    }
}
