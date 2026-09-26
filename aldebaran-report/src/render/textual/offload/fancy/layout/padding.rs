//! Module hosting multiple padding strategies.
//!
//! The list is as follows:
//! - [`HorizontalPadding`]
//! - [`VerticalPadding`]
//! - [`Padding`] (four axis padding)

/// The primitive type used to express arbitrary padding.
pub type PaddingPrimitive = u32;

/// The horizontal padding strategy.
///
/// This structure contains the padding for the left and right sides of an
/// arbitrary element.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
pub struct HorizontalPadding {
    /// Number of cells reserved on the left side.
    left: PaddingPrimitive,
    /// Number of cells reserved on the right side.
    right: PaddingPrimitive,
}

impl HorizontalPadding {
    /// Create a new [`HorizontalPadding`] with a determined left and right
    /// padding.
    #[inline]
    pub const fn pair(left: PaddingPrimitive, right: PaddingPrimitive) -> Self {
        Self { left, right }
    }

    /// Create a new [`HorizontalPadding`] from a 2-element tuple.
    #[inline]
    pub const fn tuple((left, right): (PaddingPrimitive, PaddingPrimitive)) -> Self {
        Self { left, right }
    }

    /// Create a new [`HorizontalPadding`] from a 2-element array.
    #[inline]
    pub const fn array([left, right]: [PaddingPrimitive; 2]) -> Self {
        Self { left, right }
    }

    /// Create a new [`HorizontalPadding`] with the same padding for both the
    /// left and right sides.
    ///
    /// This is equivalent to calling [`HorizontalPadding::pair`] with both
    /// arguments as `padding`.
    #[inline]
    pub const fn uniform(padding: PaddingPrimitive) -> Self {
        Self::pair(padding, padding)
    }

    /// Create a new [`HorizontalPadding`] with zeroed paddings.
    ///
    /// This is equivalent to calling [`HorizontalPadding::uniform()`], with a
    /// `0` argument.
    #[inline]
    pub const fn zeroed() -> Self {
        Self::uniform(0)
    }

    /// Retrieve the left padding of this [`HorizontalPadding`].
    #[inline]
    pub const fn left(&self) -> PaddingPrimitive {
        let &Self { left, .. } = self;

        left
    }

    /// Retrieve the right padding of this [`HorizontalPadding`].
    #[inline]
    pub const fn right(&self) -> PaddingPrimitive {
        let &Self { right, .. } = self;

        right
    }

    /// Retrieve a mutable reference to the left padding of this
    /// [`HorizontalPadding`].
    #[inline]
    pub const fn left_mut(&mut self) -> &mut PaddingPrimitive {
        let &mut Self { ref mut left, .. } = self;

        left
    }

    /// Retrieve a mutable reference to the right padding of this
    /// [`HorizontalPadding`].
    #[inline]
    pub const fn right_mut(&mut self) -> &mut PaddingPrimitive {
        let &mut Self { ref mut right, .. } = self;

        right
    }

    /// Replace the left padding of this [`HorizontalPadding`] with a new value.
    #[inline]
    pub const fn left_replace(&mut self, target_value: PaddingPrimitive) -> PaddingPrimitive {
        let &mut Self { left: previous_value, .. } = self;
        let &mut Self { ref mut left, .. } = self;

        *left = target_value;

        previous_value
    }

    /// Replace the right padding of this [`HorizontalPadding`] with a new
    /// value.
    #[inline]
    pub const fn right_replace(&mut self, target_value: PaddingPrimitive) -> PaddingPrimitive {
        let &mut Self { right: previous_value, .. } = self;
        let &mut Self { ref mut right, .. } = self;

        *right = target_value;

        previous_value
    }

    /// Determine the absolute padding of this [`HorizontalPadding`].
    ///
    /// Absolute padding is the sum of the left and right paddings.
    #[inline]
    pub const fn absolute(&self) -> PaddingPrimitive {
        let &Self { left, right } = self;

        left + right
    }
}

/// The vertical padding strategy.
///
/// This structure contains the padding for the top and bottom sides of an
/// arbitrary element.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
pub struct VerticalPadding {
    /// Number of cells reserved above the element.
    top: PaddingPrimitive,
    /// Number of cells reserved below the element.
    bottom: PaddingPrimitive,
}

impl VerticalPadding {
    /// Create a new [`VerticalPadding`] with a determined top and bottom
    /// padding.
    #[inline]
    pub const fn new(top: PaddingPrimitive, bottom: PaddingPrimitive) -> Self {
        Self { top, bottom }
    }

    /// Create a new [`VerticalPadding`] from a 2-element tuple.
    #[inline]
    pub const fn tuple((top, bottom): (PaddingPrimitive, PaddingPrimitive)) -> Self {
        Self { top, bottom }
    }

    /// Create a new [`VerticalPadding`] from a 2-element array.
    #[inline]
    pub const fn array([top, bottom]: [PaddingPrimitive; 2]) -> Self {
        Self { top, bottom }
    }

    /// Create a new [`VerticalPadding`] with the same padding for both the top
    /// and bottom sides. This is equivalent to calling
    /// [`VerticalPadding::new(padding, padding)`].
    #[inline]
    pub const fn uniform(padding: PaddingPrimitive) -> Self {
        Self::new(padding, padding)
    }

    /// Create a new [`VerticalPadding`] with zeroed paddings.
    ///
    /// This is equivalent to calling [`VerticalPadding::new(0, 0)`].
    #[inline]
    pub const fn zeroed() -> Self {
        Self::new(0, 0)
    }

    /// Retrieve the top padding of this [`VerticalPadding`].
    #[inline]
    pub const fn top(&self) -> PaddingPrimitive {
        let &Self { top, .. } = self;

        top
    }

    /// Retrieve the bottom padding of this [`VerticalPadding`].
    #[inline]
    pub const fn bottom(&self) -> PaddingPrimitive {
        let &Self { bottom, .. } = self;

        bottom
    }

    /// Retrieve a mutable reference to the top padding of this
    /// [`VerticalPadding`].
    #[inline]
    pub const fn top_mut(&mut self) -> &mut PaddingPrimitive {
        let &mut Self { ref mut top, .. } = self;

        top
    }

    /// Retrieve a mutable reference to the bottom padding of this
    /// [`VerticalPadding`].
    #[inline]
    pub const fn bottom_mut(&mut self) -> &mut PaddingPrimitive {
        let &mut Self { ref mut bottom, .. } = self;

        bottom
    }

    /// Replace the top padding of this [`VerticalPadding`] with a new value.
    #[inline]
    pub const fn top_replace(&mut self, target_value: PaddingPrimitive) -> PaddingPrimitive {
        let &mut Self { top: previous_value, .. } = self;
        let &mut Self { ref mut top, .. } = self;

        *top = target_value;

        previous_value
    }

    /// Replace the bottom padding of this [`VerticalPadding`] with a new value.
    #[inline]
    pub const fn bottom_replace(&mut self, target_value: PaddingPrimitive) -> PaddingPrimitive {
        let &mut Self {
            bottom: previous_value, ..
        } = self;
        let &mut Self { ref mut bottom, .. } = self;

        *bottom = target_value;

        previous_value
    }

    /// Determine the absolute padding of this [`VerticalPadding`].
    ///
    /// Absolute padding is the sum of the top and bottom paddings.
    #[inline]
    pub const fn absolute(&self) -> PaddingPrimitive {
        let &Self { top, bottom } = self;

        top + bottom
    }
}

/// A four-axis padding strategy.
///
/// This structure contains the padding for all four sides of an arbitrary
/// element.
///
/// Internally, this is simply a composite type of [`HorizontalPadding`] and
/// [`VerticalPadding`] structures, and provides accessors to both.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
pub struct Padding {
    /// Left and right padding.
    horizontal: HorizontalPadding,
    /// Top and bottom padding.
    vertical: VerticalPadding,
}

impl Padding {
    /// Create a new [`Padding`] with determined paddings for all four sides.
    #[inline]
    pub const fn from_raw_parts(left: PaddingPrimitive, right: PaddingPrimitive, top: PaddingPrimitive, bottom: PaddingPrimitive) -> Self {
        Self {
            horizontal: HorizontalPadding::pair(left, right),
            vertical: VerticalPadding::new(top, bottom),
        }
    }

    /// Create a new [`Padding`] with the same padding for all four sides.
    /// This is equivalent to calling [`Padding::from_raw_parts`] with the
    /// same value for all four sides.
    #[inline]
    pub const fn uniform(padding: PaddingPrimitive) -> Self {
        Self::from_raw_parts(padding, padding, padding, padding)
    }

    /// Create a new [`Padding`] with zeroed paddings.
    ///
    /// This is equivalent to calling [`Padding::uniform`].
    #[inline]
    pub const fn zeroed() -> Self {
        Self::uniform(0)
    }

    /// Retrieve the left padding of this [`Padding`].
    #[inline]
    pub const fn left(&self) -> PaddingPrimitive {
        let &Self {
            horizontal: HorizontalPadding { left, .. },
            ..
        } = self;

        left
    }

    /// Retrieve the right padding of this [`Padding`].
    #[inline]
    pub const fn right(&self) -> PaddingPrimitive {
        let &Self {
            horizontal: HorizontalPadding { right, .. },
            ..
        } = self;

        right
    }

    /// Retrieve the top padding of this [`Padding`].
    #[inline]
    pub const fn top(&self) -> PaddingPrimitive {
        let &Self {
            vertical: VerticalPadding { top, .. },
            ..
        } = self;

        top
    }

    /// Retrieve the bottom padding of this [`Padding`].
    #[inline]
    pub const fn bottom(&self) -> PaddingPrimitive {
        let &Self {
            vertical: VerticalPadding { bottom, .. },
            ..
        } = self;

        bottom
    }

    /// Retrieve a mutable reference to the left padding of this [`Padding`].
    #[inline]
    pub fn left_mut(&mut self) -> &mut PaddingPrimitive {
        let &mut Self {
            horizontal: HorizontalPadding { ref mut left, .. },
            ..
        } = self;

        left
    }

    /// Retrieve a mutable reference to the right padding of this [`Padding`].
    #[inline]
    pub fn right_mut(&mut self) -> &mut PaddingPrimitive {
        let &mut Self {
            horizontal: HorizontalPadding { ref mut right, .. },
            ..
        } = self;

        right
    }

    /// Retrieve a mutable reference to the top padding of this [`Padding`].
    #[inline]
    pub fn top_mut(&mut self) -> &mut PaddingPrimitive {
        let &mut Self {
            vertical: VerticalPadding { ref mut top, .. },
            ..
        } = self;

        top
    }

    /// Retrieve a mutable reference to the bottom padding of this [`Padding`].
    #[inline]
    pub fn bottom_mut(&mut self) -> &mut PaddingPrimitive {
        let &mut Self {
            vertical: VerticalPadding { ref mut bottom, .. },
            ..
        } = self;

        bottom
    }

    /// Replace the left padding of this [`Padding`] with a new value.
    #[inline]
    pub fn left_replace(&mut self, target_value: PaddingPrimitive) -> PaddingPrimitive {
        let &mut Self {
            horizontal: HorizontalPadding { ref mut left, .. },
            ..
        } = self;

        let previous_value = *left;

        *left = target_value;

        previous_value
    }

    /// Replace the right padding of this [`Padding`] with a new value.
    #[inline]
    pub fn right_replace(&mut self, target_value: PaddingPrimitive) -> PaddingPrimitive {
        let &mut Self {
            horizontal: HorizontalPadding { ref mut right, .. },
            ..
        } = self;

        let previous_value = *right;

        *right = target_value;

        previous_value
    }

    /// Replace the top padding of this [`Padding`] with a new value.
    #[inline]
    pub fn top_replace(&mut self, target_value: PaddingPrimitive) -> PaddingPrimitive {
        let &mut Self {
            vertical: VerticalPadding { ref mut top, .. },
            ..
        } = self;

        let previous_value = *top;

        *top = target_value;

        previous_value
    }

    /// Replace the bottom padding of this [`Padding`] with a new value.
    #[inline]
    pub fn bottom_replace(&mut self, target_value: PaddingPrimitive) -> PaddingPrimitive {
        let &mut Self {
            vertical: VerticalPadding { ref mut bottom, .. },
            ..
        } = self;

        let previous_value = *bottom;

        *bottom = target_value;

        previous_value
    }

    /// Retrieve a reference to the horizontal padding of this [`Padding`].
    #[inline]
    pub const fn horizontal(&self) -> &HorizontalPadding {
        let &Self { ref horizontal, .. } = self;

        horizontal
    }

    /// Retrieve a reference to the vertical padding of this [`Padding`].
    #[inline]
    pub const fn vertical(&self) -> &VerticalPadding {
        let &Self { ref vertical, .. } = self;

        vertical
    }

    /// Retrieve a mutable reference to the horizontal padding of this
    /// [`Padding`].
    #[inline]
    pub fn horizontal_mut(&mut self) -> &mut HorizontalPadding {
        let &mut Self { ref mut horizontal, .. } = self;

        horizontal
    }

    /// Retrieve a mutable reference to the vertical padding of this
    /// [`Padding`].
    #[inline]
    pub fn vertical_mut(&mut self) -> &mut VerticalPadding {
        let &mut Self { ref mut vertical, .. } = self;

        vertical
    }
}
