//! Styles assigned to diagnostic roles in Fancy output.
//!
//! [`Theme`] separates heading styles chosen from report metadata from line-number
//! and annotation-marker styles. [`HeaderTheme`] keeps the kind and title styles
//! together, while [`DefaultTheme`] provides the built-in palette. Rendering
//! settings control whether these styles are emitted, independently of glyphs.

use core::iter;

use aldebaran_ansi::{
    brush::style::{Attributes, TextAttribute},
    prelude::{Color, Style},
};

use crate::report::severity::{Importance, Metadata, Severity};

/// Separate styles for the diagnostic kind and the report title that follows it.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
// NOTE(invariant): None.
pub struct HeaderTheme {
    /// The style to use for the error kind.
    kind: Style,

    /// The style to use for the error title.
    title: Style,
}

impl HeaderTheme {
    /// Retrieve the style to use for the error kind.
    ///
    /// [`DefaultTheme`] chooses this style based on severity.
    #[inline]
    pub const fn kind(&self) -> Style {
        let &Self { kind, .. } = self;

        kind
    }

    /// Retrieve the style to use for the error title.
    ///
    /// [`DefaultTheme`] chooses this style based on importance.
    #[inline]
    pub const fn title(&self) -> Style {
        let &Self { title, .. } = self;

        title
    }
}

/// A trait that represents a theme for the fancy textual renderer.
///
/// Heading styles depend on report [`Metadata`]. Marker styles distinguish source
/// spans, while line-number styling is shared by the gutter and source-range
/// heading. [`Fancy`](super::Fancy) applies these styles only when its color
/// setting permits ANSI styling, but can still query the theme when it is disabled.
pub trait Theme: Copy + 'static {
    /// Determine the header theme to use for the given [`Metadata`].
    ///
    /// This will apply the stylistic settings specified depending on the
    /// error metadata, which is composed of an importance and severity
    /// level pair.
    ///
    /// See [`HeaderTheme`] and [`Metadata`] for more information.
    fn metadata(&self, metadata: &Metadata) -> HeaderTheme;

    /// Styles for distinguishing annotated source spans, with a fallback for missing styles.
    ///
    /// [`Fancy`](super::Fancy) associates styles in encounter order with original
    /// annotation spans within an excerpt and reuses the style wherever that span
    /// reappears. The fallback covers spans for which the iterator supplies no
    /// style. Marker glyphs come independently from [`super::charset::Charset`].
    #[inline]
    fn arrow(&self) -> (Style, impl Iterator<Item = Style>) {
        (Style::default(), iter::empty())
    }

    /// Determine the style to use for the line number indicators
    /// inside the source code snippet.
    ///
    /// This style also applies to the line range in the source heading.
    #[inline]
    fn lines(&self) -> Style {
        Style::default()
    }

    /// Style available for source-frame glyphs.
    ///
    /// The built-in [`Fancy`](super::Fancy) renderer does not apply this style.
    #[inline]
    fn shapes(&self) -> Style {
        Style::default()
    }
}

/// The default theme for text-based rendering.
///
/// This holds a simple, readable color scheme that is easy on the eyes.
///
/// Severity selects the kind's color and attribute, while importance selects the
/// title's. Six marker colors distinguish the first encountered spans, after
/// which markers use bright white. Line numbers retain the default style.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
pub struct DefaultTheme;

impl Theme for DefaultTheme {
    #[inline]
    fn metadata(&self, &Metadata { severity, importance }: &Metadata) -> HeaderTheme {
        let kind = {
            let (fg, attr) = match severity {
                Severity::Fatal => (Color::BRIGHT_MAGENTA, TextAttribute::Bold),
                Severity::Error => (Color::BRIGHT_RED, TextAttribute::Bold),
                Severity::Warning => (Color::YELLOW, TextAttribute::Italic),
            };

            let mut style = Style::empty();

            _ = style.foreground_replace(fg);
            _ = style.attribute_replace(Attributes::single(attr));

            style
        };

        let title = {
            let (fg, attr) = match importance {
                Importance::Critical => (Color::BRIGHT_CYAN, TextAttribute::Underline),
                Importance::Trivial => (Color::BRIGHT_WHITE, TextAttribute::Bold),
                Importance::Marginal => (Color::BRIGHT_WHITE, TextAttribute::Italic),
            };

            let mut style = Style::empty();

            _ = style.foreground_replace(fg);
            _ = style.attribute_replace(Attributes::single(attr));

            style
        };

        HeaderTheme { kind, title }
    }

    #[inline]
    fn arrow(&self) -> (Style, impl Iterator<Item = Style>) {
        let default = {
            let mut style = Style::empty();

            let _: Option<Color> = style.foreground_replace(Color::BRIGHT_WHITE);

            style
        };

        let colors = &[
            Color::BRIGHT_CYAN,
            Color::BRIGHT_MAGENTA,
            Color::BLUE,
            Color::BRIGHT_BLUE,
            Color::BRIGHT_YELLOW,
            Color::BRIGHT_WHITE,
        ];

        (
            default,
            colors.iter().map(|&color| {
                let mut style = Style::empty();

                let _: Option<Color> = style.foreground_replace(color);

                style
            }),
        )
    }
}
