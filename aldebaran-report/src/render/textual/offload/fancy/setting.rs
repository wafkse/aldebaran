//! Presentation settings for the fancy textual renderer.
//!
//! Fancyness controls how much contextual structure is shown, while color policy
//! and theme-specific settings determine styling. [`FancySettings`] carries these
//! policies together as the explicit input to the rendering backend.

use crate::render::textual::offload::fancy::{
    charset::Charset,
    layout::LayoutSettings,
    theme::{DefaultTheme, Theme},
};

/// The fancyness level for the [`Fancy`] renderer.
///
/// [`Fancy`]: super::Fancy
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
pub enum Fancyness {
    /// A colored output with unicode glyphs.
    Outblown,

    /// A colored output with ascii glyphs.
    Medium,

    /// No colors, no glyphs. For the simple folk.
    #[default]
    Meek,
}

impl Fancyness {
    /// Determine whether the current [`Fancyness`] level is colored.
    #[inline]
    pub const fn colored(&self) -> bool {
        match self {
            Self::Outblown | Self::Medium => true,
            Self::Meek => false,
        }
    }

    /// Determine whether the current [`Fancyness`] level implies unicode
    /// glyphs.
    #[inline]
    pub const fn unicode(&self) -> bool {
        match self {
            Self::Outblown => true,
            Self::Medium | Self::Meek => false,
        }
    }
}

/// Whether the output should be colored. Default to no colors.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Colored {
    /// Yes, the output should be colored.
    Yes,
    /// No, the output should not be colored.
    No,
}

impl Colored {
    /// The standard [`Colored`] setting.
    #[inline]
    pub const fn standard() -> Self {
        Self::No
    }
}

impl Colored {
    /// Create a new [`Colored`] setting from a boolean value.
    #[inline]
    pub const fn bool(value: bool) -> Self {
        if value { Self::Yes } else { Self::No }
    }

    /// Determine whether the current [`Colored`] level implies a colored
    /// output.
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

/// The settings for the [`Fancy`] offload renderer.
///
/// [`Fancy`]: super::Fancy
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct FancySettings<T = DefaultTheme>
where
    T: Theme,
{
    /// Whether rendered output uses terminal color.
    colored: Colored,

    /// Optional layout policy for source and line-number presentation.
    layout: Option<LayoutSettings>,

    /// Glyph collection used for structural and annotation markers.
    charset: Charset,

    /// Theme that maps diagnostic roles to styles.
    theme: T,
}

impl FancySettings {
    /// The standard [`FancySettings`].
    ///
    /// Uses the standard setting for each component.
    ///
    /// # Remarks
    ///
    /// This does not omit any of the settings.
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
    /// Create a new set of [`FancySettings`] from the instrinsic fancyness
    /// level.
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

    /// Create a new set of [`FancySettings`] from a 3-element tuple.
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
    /// Create a new set of [`FancySettings`] from its raw parts.
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

    /// Retrieve the colored settings of these settings.
    #[inline]
    pub const fn colored(&self) -> Colored {
        let &Self { colored, .. } = self;

        colored
    }

    /// Retrieve the layout settings of these settings.
    #[inline]
    pub const fn layout(&self) -> Option<LayoutSettings> {
        let &Self { layout, .. } = self;

        layout
    }

    /// Retrieve the charset settings of these settings.
    #[inline]
    pub const fn charset(&self) -> Charset {
        let &Self { charset, .. } = self;

        charset
    }

    /// Retrieve the theme settings of these settings.
    #[inline]
    pub const fn theme(&self) -> &T {
        let &Self { ref theme, .. } = self;

        theme
    }

    /// Retrieve *(an immutable reference)* to the colored settings of these
    /// settings.
    #[inline]
    pub const fn colored_ref(&self) -> &Colored {
        let &Self { ref colored, .. } = self;

        colored
    }

    /// Retrieve *(an immutable reference)* to the layout settings of these
    /// settings.
    #[inline]
    pub const fn layout_ref(&self) -> Option<&LayoutSettings> {
        let &Self { ref layout, .. } = self;

        layout.as_ref()
    }

    /// Retrieve *(an immutable reference)* to the charset settings of these
    /// settings.
    #[inline]
    pub const fn charset_ref(&self) -> &Charset {
        let &Self { ref charset, .. } = self;

        charset
    }

    /// Retrieve *(an immutable reference)* to the theme settings of these
    /// settings.
    #[inline]
    pub const fn theme_ref(&self) -> &T {
        let &Self { ref theme, .. } = self;

        theme
    }

    /// Retrieve *(a mutable reference)* to the colored settings of these
    /// settings.
    #[inline]
    pub const fn colored_mut(&mut self) -> &mut Colored {
        let &mut Self { ref mut colored, .. } = self;

        colored
    }

    /// Retrieve *(a mutable reference)* to the layout settings of these
    /// settings.
    #[inline]
    pub const fn layout_mut(&mut self) -> Option<&mut LayoutSettings> {
        let &mut Self { ref mut layout, .. } = self;

        layout.as_mut()
    }

    /// Retrieve *(a mutable reference)* to the charset settings of these
    /// settings.
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
