//! Gutter spacing between the margin, line numbers, separator, and source.
//!
//! See [`LayoutSettings`] and [`SourceSpacing`] for further information.
//!
//! Fancy reserves the same gutter width on source, marker, and message rows so their
//! separators align. Explicit settings take precedence over spacing derived from the
//! viewport shape.
//!
//! [`Fancy`]: super::Fancy

pub mod padding;

use padding::HorizontalPadding;

use core::{fmt, num::NonZero};

use aldebaran_print::prelude::{Combine, Print};

use crate::render::textual::viewport::{self, Viewport, ViewportShape};

use super::run::GlyphRun;

/// Spacing immediately after the gutter separator and before source or markers.
///
/// Tabs expand to spaces, with their stops measured from immediately after the
/// separator. Their configured width is independent of the destination's tab
/// stops and of tabs within source text.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum SourceSpacing {
    /// A nonzero number of spaces.
    Spaces {
        /// Number of spaces to emit.
        amount: NonZero<u32>,
    },

    /// A nonzero number of tabs, each occupying the configured number of cells.
    Tabs {
        /// Number of spaces occupied by each tab.
        width: NonZero<u32>,

        /// Number of tabs to expand.
        amount: NonZero<u32>,
    },
}

impl Default for SourceSpacing {
    #[inline]
    fn default() -> Self {
        Self::Spaces {
            amount: NonZero::<u32>::MIN,
        }
    }
}

impl Print for SourceSpacing {
    type Context = ();

    #[inline]
    fn print_with_context<W>(&self, writer: &mut W, _: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        match *self {
            Self::Spaces { amount } => GlyphRun {
                glyph: ' ',
                count: amount.get(),
            }
            .print(writer),
            Self::Tabs { width, amount } => GlyphRun {
                glyph: ' ',
                count: width.get(),
            }
            .times(amount.get())
            .print(writer),
        }
    }
}

/// Spacing around [`Fancy`]'s line-number digits and after the source separator.
///
/// The digit column is sized to the largest displayed line number. These spaces
/// widen that gutter without reducing the viewport width used for source
/// wrapping. [`Self::viewport`] chooses padding from the layout preference,
/// while [`Default`] uses no padding.
///
/// [`Fancy`]: super::Fancy
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
// NOTE(invariant): None.
pub struct LayoutSettings {
    /// Spaces before the digit column and between the digits and source separator.
    line_no_col_padding: HorizontalPadding,

    /// Shared indentation for source, marker, and message rows after the separator.
    source_spacing: SourceSpacing,
}

impl LayoutSettings {
    /// One space before the digit column and two before the separator.
    pub const STANDARD: Self = Self::standard();

    /// Gutter width determined solely by the line-number digits.
    pub const MARGINAL: Self = Self::marginal();

    /// Retrieve the standard layout settings.
    ///
    /// This is the same as calling [`LayoutSettings::viewport`] with a
    /// [`standard viewport`](Viewport::standard).
    #[inline]
    pub const fn standard() -> Self {
        Self::viewport(&Viewport::standard())
    }

    /// Choose gutter spacing from the viewport's layout preference.
    ///
    /// Squarish viewports use no padding, rightward preference adds one space on
    /// the left and two on the right, and downward preference adds one on each
    /// side. Explicit shape overrides apply even when the dimensions disagree.
    #[inline]
    pub const fn viewport(viewport: &Viewport) -> Self {
        match viewport.shape() {
            ViewportShape::Squarish => Self::marginal(),
            ViewportShape::Rectangular(target_trend) => match target_trend {
                viewport::Trend::Rightwards => Self::raw(HorizontalPadding::tuple((1, 2))),
                viewport::Trend::Downwards => Self::raw(HorizontalPadding::uniform(1)),
            },
        }
    }
}

impl LayoutSettings {
    /// Keep only the width needed for the line-number digits, as [`Default`] does.
    #[inline]
    pub const fn marginal() -> Self {
        let line_no_col_padding = HorizontalPadding::uniform(u32::MIN);

        Self::raw(line_no_col_padding)
    }

    /// Fix the spaces before and after the digits independently of viewport shape.
    #[inline]
    pub const fn raw(line_no_col_padding: HorizontalPadding) -> Self {
        let source_spacing = SourceSpacing::Spaces {
            amount: NonZero::<u32>::MIN,
        };

        Self {
            line_no_col_padding,
            source_spacing,
        }
    }

    /// Retrieve the horizontal padding for the line column of the source excerpt.
    #[inline]
    pub const fn line_column_padding(&self) -> HorizontalPadding {
        let &Self { line_no_col_padding, .. } = self;

        line_no_col_padding
    }

    /// Indentation immediately after the separator, defaulting to one space.
    #[inline]
    pub const fn source_spacing(&self) -> SourceSpacing {
        let &Self { source_spacing, .. } = self;

        source_spacing
    }

    /// Change indentation shared by source, marker, and message rows.
    #[inline]
    pub const fn source_spacing_mut(&mut self) -> &mut SourceSpacing {
        let &mut Self {
            ref mut source_spacing, ..
        } = self;

        source_spacing
    }
}
