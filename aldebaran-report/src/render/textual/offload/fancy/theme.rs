//! Styling policy for fancy textual diagnostics.
//!
//! [`Theme`] supplies styles for headers, source lines, annotation markers, and
//! related rendering roles. The renderer consumes these semantic styles without
//! depending on a concrete color scheme, and [`DefaultTheme`] provides defaults.

use aldebaran_ansi::{
    brush::style::{Attributes, TextAttribute},
    prelude::{Color, Style},
};

use crate::report::severity::{Importance, Metadata, Severity};

/// A theme construct for error report headers.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct HeaderTheme {
    /// The style to use for the error kind.
    kind: Style,

    /// The style to use for the error title.
    title: Style,
}

impl HeaderTheme {
    /// Retrieve the style to use for the error kind.
    #[inline]
    pub const fn kind(&self) -> Style {
        let &Self { kind, .. } = self;

        kind
    }

    /// Retrieve the style to use for the error title.
    #[inline]
    pub const fn title(&self) -> Style {
        let &Self { title, .. } = self;

        title
    }
}

/// A trait that represents a theme for the fancy textual renderer.
pub trait Theme: Copy + 'static {
    /// Determine the header theme to use for the given [`Metadata`].
    ///
    /// This will apply the stylistic settings specified depending on the
    /// error metadata, which is composed of an importance and severity
    /// level pair.
    ///
    /// See [`HeaderTheme`] and [`Metadata`] for more information.
    fn metadata(&self, metadata: &Metadata) -> HeaderTheme;

    /// Determine the set of styles to use for the arrow glyphs.
    ///
    /// The arrow glyphs serve as a visual indicator, so the styles
    /// ought to be distinct and noticeable.
    ///
    /// The iterator is deemed to be of an arbitrary size, and will
    /// default to the first style if the iterator yields no more.
    #[inline]
    fn arrow(&self) -> (Style, impl Iterator<Item = Style>) {
        (Style::default(), core::iter::empty())
    }

    /// Determine the style to use for the line number indicators
    /// inside the source code snippet.
    #[inline]
    fn lines(&self) -> Style {
        Style::default()
    }

    /// Determine the style to use for the shapes that enclose the
    /// source code snippet.
    #[inline]
    fn shapes(&self) -> Style {
        Style::default()
    }
}

/// The default theme for text-based rendering.
///
/// This holds a simple, readable color scheme that is easy on the eyes.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
pub struct DefaultTheme;

impl Theme for DefaultTheme {
    fn metadata(&self, &Metadata { severity, importance }: &Metadata) -> HeaderTheme {
        let kind = {
            let (fg, attr) = match severity {
                Severity::Fatal => (Color::BRIGHT_MAGENTA, TextAttribute::Bold),
                Severity::Error => (Color::BRIGHT_RED, TextAttribute::Bold),
                Severity::Warning => (Color::YELLOW, TextAttribute::Italic),
            };

            let mut style = Style::empty();

            let _ = style.foreground_replace(fg);
            let _ = style.attribute_replace(Attributes::single(attr));

            style
        };

        let title = {
            let (fg, attr) = match importance {
                Importance::Critical => (Color::BRIGHT_CYAN, TextAttribute::Underline),
                Importance::Trivial => (Color::BRIGHT_WHITE, TextAttribute::Bold),
                Importance::Marginal => (Color::BRIGHT_WHITE, TextAttribute::Italic),
            };

            let mut style = Style::empty();

            let _ = style.foreground_replace(fg);
            let _ = style.attribute_replace(Attributes::single(attr));

            style
        };

        HeaderTheme { kind, title }
    }

    fn arrow(&self) -> (Style, impl Iterator<Item = Style>) {
        let default = {
            let mut style = Style::empty();

            let _ = style.foreground_replace(Color::BRIGHT_WHITE);

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

                let _ = style.foreground_replace(color);

                style
            }),
        )
    }
}
