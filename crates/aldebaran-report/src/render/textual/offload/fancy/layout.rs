//! Gutter spacing between the margin, line numbers, and source separator.
//!
//! [`LayoutSettings`] uses [`padding::HorizontalPadding`] to add spaces around
//! the digit column. [`Fancy`] reserves the same gutter width on source, marker,
//! and message rows so their separators align. Explicit settings take precedence
//! over spacing derived from the [`Viewport`] shape.
//!
//! [`Fancy`]: super::Fancy

pub mod padding;

use padding::HorizontalPadding;

use crate::render::textual::viewport::{self, Viewport, ViewportShape};

/// Extra spaces on either side of [`Fancy`]'s line-number digits.
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
}

impl LayoutSettings {
    /// One space before the digit column and two before the separator.
    pub const STANDARD: Self = Self::standard();

    /// Gutter width determined solely by the line-number digits.
    pub const MARGINAL: Self = Self::marginal();

    /// Use the roomier rightward spacing of [`Viewport::standard`].
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
        let line_no_col_padding = HorizontalPadding::uniform(0);

        Self { line_no_col_padding }
    }

    /// Fix the spaces before and after the digits independently of viewport shape.
    #[inline]
    pub const fn raw(line_no_col_padding: HorizontalPadding) -> Self {
        Self { line_no_col_padding }
    }

    /// Spaces before the digit column and before the source separator.
    #[inline]
    pub const fn line_column_padding(&self) -> HorizontalPadding {
        let &Self { line_no_col_padding, .. } = self;

        line_no_col_padding
    }
}
