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
    /// Style of the diagnostic kind descriptor before the heading separator.
    kind: Style,

    /// Style of the report title following the kind descriptor.
    title: Style,
}

impl HeaderTheme {
    /// Heading emphasis for the kind descriptor, which [`DefaultTheme`] bases on severity.
    #[inline]
    pub const fn kind(&self) -> Style {
        let &Self { kind, .. } = self;

        kind
    }

    /// Heading emphasis for the report title, which [`DefaultTheme`] bases on importance.
    #[inline]
    pub const fn title(&self) -> Style {
        let &Self { title, .. } = self;

        title
    }
}

/// Assigns styles to the heading and annotated source excerpt.
///
/// Heading styles depend on report [`Metadata`]. Marker styles distinguish source
/// spans, while line-number styling is shared by the gutter and source-range
/// heading. [`Fancy`](super::Fancy) applies these styles only when its color
/// setting permits ANSI styling, but can still query the theme when it is disabled.
pub trait Theme: Copy + 'static {
    /// Choose separate kind and title styles from the report's severity and importance.
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

    /// Shared style for gutter line numbers and the source heading's line range.
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

/// Built-in emphasis for report severity, title importance, and annotation spans.
///
/// Severity selects the kind's color and attribute, while importance selects the
/// title's. Six marker colors distinguish the first encountered spans, after
/// which markers use bright white. Line numbers retain the default style.
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
