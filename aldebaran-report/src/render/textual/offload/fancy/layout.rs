//! Module for the layout settings of the [`Fancy`] offload renderer and
//! adjacent types.
//!
//! [`Fancy`]: super::Fancy

pub mod padding;

use padding::HorizontalPadding;

use crate::render::textual::viewport::{self, Viewport, ViewportShape};

/// The layout settings for the [`Fancy`] offload renderer.
///
/// [`Fancy`]: super::Fancy
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
pub struct LayoutSettings {
    line_no_col_padding: HorizontalPadding,
}

impl LayoutSettings {
    /// The standard layout settings.
    ///
    /// This constant is the same as calling [`LayoutSettings::standard()`].
    pub const STANDARD: Self = Self::standard();

    /// The marginal layout settings.
    ///
    /// This constant is the same as calling [`LayoutSettings::marginal()`].
    ///
    /// This is the narrowest acceptable layout settings that allow for a
    /// readable output.
    pub const MARGINAL: Self = Self::marginal();

    /// Retrieve the standard layout settings.
    ///
    /// This is the same as calling [`LayoutSettings::viewport`] with a
    /// [`standard viewport`](Viewport::standard).
    #[inline]
    pub const fn standard() -> Self {
        Self::viewport(&Viewport::standard())
    }

    /// Create a new set of [`LayoutSettings`] with the appropiate padding for
    /// the target [`Viewport`] configuration.
    #[inline]
    pub const fn viewport(viewport: &Viewport) -> Self {
        match viewport.shape() {
            ViewportShape::Squarish => Self::marginal(),
            ViewportShape::Rectangular(target_trend) => match target_trend {
                viewport::Trend::Rightwards => Self::from_raw_parts(HorizontalPadding::tuple((1, 2))),
                viewport::Trend::Downwards => Self::from_raw_parts(HorizontalPadding::uniform(1)),
            },
        }
    }
}

impl LayoutSettings {
    /// Create a new set of [`LayoutSettings`] with the marginal padding
    /// configuration.
    #[inline]
    pub const fn marginal() -> Self {
        let line_no_col_padding = HorizontalPadding::uniform(0);

        Self { line_no_col_padding }
    }

    /// Create a new set of [`LayoutSettings`] from its raw parts.
    #[inline]
    pub const fn from_raw_parts(line_no_col_padding: HorizontalPadding) -> Self {
        Self { line_no_col_padding }
    }

    /// Retrieve the horizontal padding for the line column of the code block.
    #[inline]
    pub const fn line_column_padding(&self) -> HorizontalPadding {
        let &Self { line_no_col_padding, .. } = self;

        line_no_col_padding
    }
}
