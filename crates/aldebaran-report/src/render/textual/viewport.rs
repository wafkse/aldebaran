//! Terminal viewport characteristics.
//!
//! This module contains various structures and types that are used to represent
//! the viewport of a terminal, that is, the area of the terminal that is
//! suitable for rendering text.
//!
//! The main structure of this module is the [`Viewport`] structure, which is
//! used to represent the extents of a terminal window.

use core::num::NonZero;

/// A terminal viewport.
///
/// Represents the area of the terminal that is suitable for rendering text.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Viewport {
    /// Physical width and height of the terminal viewport.
    viewport_extents: Extents,

    /// Shape classification used to choose a rendering growth direction.
    viewport_shape: ViewportShape,
}

impl Viewport {
    /// The standard terminal viewport.
    ///
    /// This constant is the same as calling [`Viewport::standard()`].
    pub const STANDARD: Self = Self::standard();

    /// Create a new [`Viewport`] with the standard terminal size.
    ///
    /// The current standard terminal size is `140x48`.
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

    /// Create a new [`Viewport`] from its [`Extents`] part, and determine the
    /// shape from it.
    #[inline]
    pub const fn from_extents(extents: Extents) -> Self {
        let viewport_shape = extents.shape();

        Self {
            viewport_extents: extents,
            viewport_shape,
        }
    }

    /// Create a new [`Viewport`] from its raw parts: [`Extents`] and
    /// [`ViewportShape`].
    #[inline]
    pub const fn from_raw_parts(extents: Extents, shape: ViewportShape) -> Self {
        Self {
            viewport_extents: extents,
            viewport_shape: shape,
        }
    }

    /// Retrieve the physical extents of this [`Viewport`].
    #[inline]
    pub const fn extents(&self) -> Extents {
        let &Self {
            viewport_extents: terminal_extents,
            ..
        } = self;

        terminal_extents
    }

    /// Retrieve the shape of this [`Viewport`].
    #[inline]
    pub const fn shape(&self) -> ViewportShape {
        let &Self {
            viewport_shape: terminal_shape,
            ..
        } = self;

        terminal_shape
    }

    /// Determine the prefered growth trend of rendered text for this
    /// [`Viewport`].
    #[inline]
    pub const fn trend(&self) -> Option<Trend> {
        match self.shape() {
            ViewportShape::Rectangular(trend) => Some(trend),
            _ => None,
        }
    }

    /// Reinterpret this [`Viewport`] as a really wide viewport.
    ///
    /// This effectively changes the shape of the viewport to be rectangular
    /// with a [`rightwards growth trend`](Trend::Rightwards).
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

    /// Reinterpret this [`Viewport`] as a really tall viewport.
    ///
    /// This effectively changes the shape of the viewport to be rectangular
    /// with a [`downwards growth trend`](Trend::Downwards).
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

/// The approximate physical shape of a viewport.
///
/// This is used to determine the appropriate course-of-action when rendering
/// text that can be overly wide or tall.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum ViewportShape {
    /// A squarish viewport.
    ///
    /// This configuration will prefer to render text in a square-like shape.
    ///
    /// The most common occurence is line-wrapping of text.
    Squarish,

    /// A rectangular viewport.
    ///
    /// While for an arbitrary viewport it is always a rectangle, this
    /// configuration will prefer to use horizontal space over vertical
    /// space.
    ///
    /// Note that line-wrapping is still possible in this configuration.
    ///
    /// This contains a [`Trend`] that indicates the prefered growth trend of
    /// rendered text for the rectangular viewport.
    Rectangular(
        /// Preferred direction for using the longer viewport axis.
        Trend,
    ),
}

impl Default for ViewportShape {
    #[inline]
    fn default() -> Self {
        Self::Rectangular(Default::default())
    }
}

/// The prefered growth trend of rendered text to fit a viewport.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
pub enum Trend {
    /// Text should grow rightwards, i.e make use of the maximum horizontal
    /// extents of the viewport.
    #[default]
    Rightwards,

    /// Text should grow downwards, i.e make use of the maximum vertical
    /// extents of the viewport.
    Downwards,
}

/// A width and height pair.
///
/// Represents the extents of a terminal window.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Extents {
    /// Nonzero viewport width in terminal cells.
    width: NonZero<u16>,

    /// Nonzero viewport height in terminal cells.
    height: NonZero<u16>,
}

impl Extents {
    /// Create a new [`Extents`] with a determined width and height tuple pair.
    #[inline]
    pub const fn tuple((width, height): (NonZero<u16>, NonZero<u16>)) -> Self {
        Self { width, height }
    }

    /// Create a new [`Extents`] from a 2-element array.
    #[inline]
    pub const fn array([width, height]: [NonZero<u16>; 2]) -> Self {
        Self { width, height }
    }

    /// Determine the general shape of these [`Extents`].
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

    /// Retrieve the width component of these [`Extents`].
    #[inline]
    pub const fn width(&self) -> u16 {
        let &Self { width, .. } = self;

        width.get()
    }

    /// Retrieve the height component of these [`Extents`].
    #[inline]
    pub const fn height(&self) -> u16 {
        let &Self { height, .. } = self;

        height.get()
    }

    /// Retrieve the width component of these [`Extents`].
    ///
    /// This is the raw width value, as per is it stored internally.
    #[inline]
    pub const fn width_raw(&self) -> NonZero<u16> {
        let &Self { width, .. } = self;

        width
    }

    /// Retrieve the height component of these [`Extents`].
    ///
    /// This is the raw height value, as per is it stored internally.
    #[inline]
    pub const fn height_raw(&self) -> NonZero<u16> {
        let &Self { height, .. } = self;

        height
    }
}
