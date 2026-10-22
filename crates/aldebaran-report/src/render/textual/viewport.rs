//! Terminal dimensions and the layout preference carried with a textual render.
//!
//! [`Extents`] measures available columns and rows in terminal cells, while
//! [`Viewport`] pairs those dimensions with a [`ViewportShape`]. The shape normally
//! follows the longer axis but can be overridden independently.
//! [`Fancy`](super::offload::fancy::Fancy) uses width to wrap source content and
//! shape to select gutter padding when no explicit layout is present. Height
//! does not limit the number of output rows.

use core::num::NonZero;

/// Available terminal dimensions with an independent layout preference.
///
/// [`Self::new`] derives the preference from the dimensions.
/// [`Self::wide`] and [`Self::tall`] override it without changing the wrapping
/// width, so layout policy need not follow the terminal's proportions.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
// NOTE(invariant): None.
pub struct Viewport {
    /// Available columns and rows, with width used as the source wrapping budget.
    viewport_extents: Extents,

    /// Layout preference, which may differ from the proportions of `viewport_extents`.
    viewport_shape: ViewportShape,
}

impl Viewport {
    /// A 140-column, 48-row viewport with a rightward layout preference.
    pub const STANDARD: Self = Self::standard();

    /// Use 140 columns and 48 rows with the corresponding rightward preference.
    #[inline]
    pub const fn standard() -> Self {
        let viewport_extents = Extents::tuple((
            NonZero::<u16>::MIN.saturating_add(140 - 1),
            NonZero::<u16>::MIN.saturating_add(48 - 1),
        ));

        let viewport_shape = viewport_extents.shape();

        Self {
            viewport_extents,
            viewport_shape,
        }
    }

    /// Let the dimensions determine the layout preference through [`Extents::shape`].
    #[inline]
    pub const fn new(extents: Extents) -> Self {
        let viewport_shape = extents.shape();

        Self {
            viewport_extents: extents,
            viewport_shape,
        }
    }

    /// Combine [`Extents`] with an explicit [`ViewportShape`].
    ///
    /// The explicit shape takes precedence even when it disagrees with the extents,
    /// allowing callers to supply their own layout classification. When the shape
    /// should follow the dimensions instead, use [`Viewport::new`].
    #[inline]
    pub const fn raw(extents: Extents, shape: ViewportShape) -> Self {
        Self {
            viewport_extents: extents,
            viewport_shape: shape,
        }
    }

    /// Available columns and rows, independent of any shape override.
    #[inline]
    pub const fn extents(&self) -> Extents {
        let &Self {
            viewport_extents: terminal_extents,
            ..
        } = self;

        terminal_extents
    }

    /// Layout preference used by viewport-derived gutter spacing.
    #[inline]
    pub const fn shape(&self) -> ViewportShape {
        let &Self {
            viewport_shape: terminal_shape,
            ..
        } = self;

        terminal_shape
    }

    /// Preferred layout axis, including any explicit override.
    ///
    /// Squarish shapes have no preferred direction and return `None`.
    #[inline]
    pub const fn trend(&self) -> Option<Trend> {
        match self.shape() {
            ViewportShape::Rectangular(trend) => Some(trend),
            _ => None,
        }
    }

    /// Select rightward gutter spacing without changing the available dimensions.
    #[inline]
    pub const fn wide(self) -> Self {
        let Self {
            viewport_extents: extents, ..
        } = self;

        let shape = ViewportShape::Rectangular(Trend::Rightwards);

        Self {
            viewport_extents: extents,
            viewport_shape: shape,
        }
    }

    /// Select downward gutter spacing without changing the available dimensions.
    #[inline]
    pub const fn tall(self) -> Self {
        let Self {
            viewport_extents: extents, ..
        } = self;

        let shape = ViewportShape::Rectangular(Trend::Downwards);

        Self {
            viewport_extents: extents,
            viewport_shape: shape,
        }
    }
}

/// Layout preference for choosing spacing from viewport proportions.
///
/// [`Extents::shape`] classifies equal dimensions as [`Self::Squarish`] and
/// otherwise favors the longer axis. An explicit [`Viewport`] override can use
/// either preference regardless of the dimensions.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum ViewportShape {
    /// No preferred axis, giving Fancy its minimal gutter padding.
    Squarish,

    /// A preferred axis for choosing roomier gutter padding.
    Rectangular(
        /// Axis favored by the layout policy, not necessarily the longer dimension.
        Trend,
    ),
}

impl Default for ViewportShape {
    #[inline]
    fn default() -> Self {
        Self::Rectangular(Default::default())
    }
}

/// Axis favored when selecting a rectangular viewport's layout.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
pub enum Trend {
    /// Favor horizontal space, with wider right padding in Fancy's gutter.
    #[default]
    Rightwards,

    /// Favor vertical space, with symmetric padding in Fancy's gutter.
    Downwards,
}

/// Available terminal columns and rows, each at least one cell.
///
/// These dimensions describe display space rather than source bytes or character
/// counts. [`Self::shape`] compares the counts directly to derive a layout axis.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
// NOTE(invariant): None.
pub struct Extents {
    /// Nonzero viewport width in terminal cells.
    width: NonZero<u16>,

    /// Nonzero viewport height in terminal cells.
    height: NonZero<u16>,
}

impl Extents {
    /// Interpret the pair as terminal columns followed by rows.
    #[inline]
    pub const fn tuple((width, height): (NonZero<u16>, NonZero<u16>)) -> Self {
        Self { width, height }
    }

    /// Interpret the entries as terminal columns followed by rows.
    #[inline]
    pub const fn array([width, height]: [NonZero<u16>; 2]) -> Self {
        Self { width, height }
    }

    /// Classify equal dimensions as squarish and otherwise follow the longer axis.
    #[inline]
    pub const fn shape(&self) -> ViewportShape {
        let &Self { width, height } = self;

        let (width, height) = (width.get(), height.get());

        if width == height {
            ViewportShape::Squarish
        } else {
            let trend = if width > height { Trend::Rightwards } else { Trend::Downwards };

            ViewportShape::Rectangular(trend)
        }
    }

    /// Terminal columns available for each source row before wrapping.
    #[inline]
    pub const fn width(&self) -> u16 {
        let &Self { width, .. } = self;

        width.get()
    }

    /// Available terminal rows, used for shape classification rather than clipping.
    #[inline]
    pub const fn height(&self) -> u16 {
        let &Self { height, .. } = self;

        height.get()
    }

    /// Column budget with its guarantee of at least one terminal cell.
    #[inline]
    pub const fn width_raw(&self) -> NonZero<u16> {
        let &Self { width, .. } = self;

        width
    }

    /// Row count with its guarantee of at least one terminal cell.
    #[inline]
    pub const fn height_raw(&self) -> NonZero<u16> {
        let &Self { height, .. } = self;

        height
    }
}
