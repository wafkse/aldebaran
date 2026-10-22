//! Padding counts grouped by axis or by all four edges.
//!
//! [`HorizontalPadding`] holds leading and trailing space counts, which
//! [`Fancy`](super::super::Fancy) uses around its line-number digit column.
//! [`VerticalPadding`] holds independent amounts above and below an element,
//! and [`Padding`] combines both axes.
//! Fancy currently consumes only horizontal padding, so it does not assign
//! rendering units to the vertical counts. All defaults reserve no padding.

/// Nonnegative edge count, interpreted as ASCII spaces for horizontal padding.
pub type PaddingPrimitive = u32;

/// Leading and trailing spaces reserved around rendered content.
///
/// Fancy places these spaces before and after the line-number digit column.
/// The counts are independent, allowing the outer margin to differ from the gap
/// before the source separator. [`Self::absolute`] gives their combined width,
/// excluding the content itself.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
// NOTE(invariant): None.
pub struct HorizontalPadding {
    /// Leading spaces before the padded content, outside its alignment width.
    left: PaddingPrimitive,
    /// Trailing spaces between the padded content and its following separator.
    right: PaddingPrimitive,
}

impl HorizontalPadding {
    /// Set the outer margin and trailing gap as independent space counts.
    #[inline]
    pub const fn pair(left: PaddingPrimitive, right: PaddingPrimitive) -> Self {
        Self { left, right }
    }

    /// Order space counts as left margin followed by right gap.
    #[inline]
    pub const fn tuple((left, right): (PaddingPrimitive, PaddingPrimitive)) -> Self {
        Self { left, right }
    }

    /// Use left and right space counts in the same order as [`Self::tuple`].
    #[inline]
    pub const fn array([left, right]: [PaddingPrimitive; 2]) -> Self {
        Self { left, right }
    }

    /// Reserve the given number of spaces on each side, doubling it in the total width.
    #[inline]
    pub const fn uniform(padding: PaddingPrimitive) -> Self {
        Self::pair(padding, padding)
    }

    /// Reserve only the content width, with no leading or trailing spaces.
    #[inline]
    pub const fn zeroed() -> Self {
        Self::uniform(0)
    }

    /// Leading space count before the content's alignment column.
    #[inline]
    pub const fn left(&self) -> PaddingPrimitive {
        let &Self { left, .. } = self;

        left
    }

    /// Trailing space count between content and a following separator.
    #[inline]
    pub const fn right(&self) -> PaddingPrimitive {
        let &Self { right, .. } = self;

        right
    }

    /// Adjust the outer margin in spaces without changing the trailing gap.
    #[inline]
    pub const fn left_mut(&mut self) -> &mut PaddingPrimitive {
        let &mut Self { ref mut left, .. } = self;

        left
    }

    /// Adjust the trailing gap in spaces without changing the outer margin.
    #[inline]
    pub const fn right_mut(&mut self) -> &mut PaddingPrimitive {
        let &mut Self { ref mut right, .. } = self;

        right
    }

    /// Replace the leading space count, yielding the previous margin for restoration.
    #[inline]
    pub const fn left_replace(&mut self, target_value: PaddingPrimitive) -> PaddingPrimitive {
        let &mut Self { ref mut left, .. } = self;

        let previous_value = *left;

        *left = target_value;

        previous_value
    }

    /// Replace the trailing space count, yielding the previous gap for restoration.
    #[inline]
    pub const fn right_replace(&mut self, target_value: PaddingPrimitive) -> PaddingPrimitive {
        let &mut Self { ref mut right, .. } = self;

        let previous_value = *right;

        *right = target_value;

        previous_value
    }

    /// Combined leading and trailing width in spaces, excluding content width.
    ///
    /// A total above [`u32::MAX`] follows the configured integer-overflow behavior.
    #[inline]
    pub const fn absolute(&self) -> PaddingPrimitive {
        let &Self { left, right } = self;

        left + right
    }
}

/// Independent padding amounts above and below an element.
///
/// These counts can accompany horizontal padding in [`Padding`], but the current
/// Fancy renderer does not consume them. A consumer determines their vertical
/// unit. [`Self::absolute`] combines the amounts without including content height.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
// NOTE(invariant): None.
pub struct VerticalPadding {
    /// Padding amount above the element in the consumer's vertical unit.
    top: PaddingPrimitive,
    /// Padding amount below the element in the same unit as `top`.
    bottom: PaddingPrimitive,
}

impl VerticalPadding {
    /// Assign the amounts above and below the content independently.
    #[inline]
    pub const fn new(top: PaddingPrimitive, bottom: PaddingPrimitive) -> Self {
        Self { top, bottom }
    }

    /// Order padding amounts as above followed by below.
    #[inline]
    pub const fn tuple((top, bottom): (PaddingPrimitive, PaddingPrimitive)) -> Self {
        Self { top, bottom }
    }

    /// Use top and bottom amounts in the same order as [`Self::tuple`].
    #[inline]
    pub const fn array([top, bottom]: [PaddingPrimitive; 2]) -> Self {
        Self { top, bottom }
    }

    /// Reserve equal amounts above and below, doubling the amount in the total.
    #[inline]
    pub const fn uniform(padding: PaddingPrimitive) -> Self {
        Self::new(padding, padding)
    }

    /// Add no padding to either end of the vertical axis.
    #[inline]
    pub const fn zeroed() -> Self {
        Self::new(0, 0)
    }

    /// Padding above the content in the consumer's vertical unit.
    #[inline]
    pub const fn top(&self) -> PaddingPrimitive {
        let &Self { top, .. } = self;

        top
    }

    /// Padding below the content in the same unit as [`Self::top`].
    #[inline]
    pub const fn bottom(&self) -> PaddingPrimitive {
        let &Self { bottom, .. } = self;

        bottom
    }

    /// Adjust space above the element independently of the lower margin.
    #[inline]
    pub const fn top_mut(&mut self) -> &mut PaddingPrimitive {
        let &mut Self { ref mut top, .. } = self;

        top
    }

    /// Adjust space below the element independently of the upper margin.
    #[inline]
    pub const fn bottom_mut(&mut self) -> &mut PaddingPrimitive {
        let &mut Self { ref mut bottom, .. } = self;

        bottom
    }

    /// Replace the upper padding amount, yielding the previous value for restoration.
    #[inline]
    pub const fn top_replace(&mut self, target_value: PaddingPrimitive) -> PaddingPrimitive {
        let &mut Self { ref mut top, .. } = self;

        let previous_value = *top;

        *top = target_value;

        previous_value
    }

    /// Replace the lower padding amount, yielding the previous value for restoration.
    #[inline]
    pub const fn bottom_replace(&mut self, target_value: PaddingPrimitive) -> PaddingPrimitive {
        let &mut Self { ref mut bottom, .. } = self;

        let previous_value = *bottom;

        *bottom = target_value;

        previous_value
    }

    /// Combined upper and lower padding, excluding the content height.
    ///
    /// A total above [`u32::MAX`] follows the configured integer-overflow behavior.
    #[inline]
    pub const fn absolute(&self) -> PaddingPrimitive {
        let &Self { top, bottom } = self;

        top + bottom
    }
}

/// Padding amounts around all four edges of an element.
///
/// [`HorizontalPadding`] groups leading and trailing spaces, while
/// [`VerticalPadding`] groups the upper and lower amounts in a consumer-defined
/// unit. The axes can be edited independently. [`Self::uniform`] assigns equal
/// numeric counts to every edge without converting between axis units.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
// NOTE(invariant): None.
pub struct Padding {
    /// Leading and trailing space counts, excluding the content width.
    horizontal: HorizontalPadding,
    /// Upper and lower amounts in the vertical consumer's unit.
    vertical: VerticalPadding,
}

impl Padding {
    /// Order edge amounts as left, right, top, and bottom.
    ///
    /// Horizontal amounts count spaces. Vertical units are determined by the consumer.
    #[inline]
    pub const fn raw(left: PaddingPrimitive, right: PaddingPrimitive, top: PaddingPrimitive, bottom: PaddingPrimitive) -> Self {
        Self {
            horizontal: HorizontalPadding::pair(left, right),
            vertical: VerticalPadding::new(top, bottom),
        }
    }

    /// Assign the same numeric count to every edge without converting between axis units.
    #[inline]
    pub const fn uniform(padding: PaddingPrimitive) -> Self {
        Self::raw(padding, padding, padding, padding)
    }

    /// Leave content extents unchanged by reserving no edge padding.
    #[inline]
    pub const fn zeroed() -> Self {
        Self::uniform(0)
    }

    /// Leading horizontal padding measured in spaces.
    #[inline]
    pub const fn left(&self) -> PaddingPrimitive {
        let &Self {
            horizontal: HorizontalPadding { left, .. },
            ..
        } = self;

        left
    }

    /// Trailing horizontal padding measured in spaces.
    #[inline]
    pub const fn right(&self) -> PaddingPrimitive {
        let &Self {
            horizontal: HorizontalPadding { right, .. },
            ..
        } = self;

        right
    }

    /// Upper padding amount in the consumer's vertical unit.
    #[inline]
    pub const fn top(&self) -> PaddingPrimitive {
        let &Self {
            vertical: VerticalPadding { top, .. },
            ..
        } = self;

        top
    }

    /// Lower padding amount in the same unit as [`Self::top`].
    #[inline]
    pub const fn bottom(&self) -> PaddingPrimitive {
        let &Self {
            vertical: VerticalPadding { bottom, .. },
            ..
        } = self;

        bottom
    }

    /// Adjust leading spaces while retaining the other edge amounts.
    #[inline]
    pub const fn left_mut(&mut self) -> &mut PaddingPrimitive {
        let &mut Self {
            horizontal: HorizontalPadding { ref mut left, .. },
            ..
        } = self;

        left
    }

    /// Adjust trailing spaces while retaining the other edge amounts.
    #[inline]
    pub const fn right_mut(&mut self) -> &mut PaddingPrimitive {
        let &mut Self {
            horizontal: HorizontalPadding { ref mut right, .. },
            ..
        } = self;

        right
    }

    /// Adjust the upper margin independently of horizontal spacing.
    #[inline]
    pub const fn top_mut(&mut self) -> &mut PaddingPrimitive {
        let &mut Self {
            vertical: VerticalPadding { ref mut top, .. },
            ..
        } = self;

        top
    }

    /// Adjust the lower margin independently of horizontal spacing.
    #[inline]
    pub const fn bottom_mut(&mut self) -> &mut PaddingPrimitive {
        let &mut Self {
            vertical: VerticalPadding { ref mut bottom, .. },
            ..
        } = self;

        bottom
    }

    /// Replace the leading space count, yielding the previous amount for restoration.
    #[inline]
    pub const fn left_replace(&mut self, target_value: PaddingPrimitive) -> PaddingPrimitive {
        let &mut Self {
            horizontal: HorizontalPadding { ref mut left, .. },
            ..
        } = self;

        let previous_value = *left;

        *left = target_value;

        previous_value
    }

    /// Replace the trailing space count, yielding the previous amount for restoration.
    #[inline]
    pub const fn right_replace(&mut self, target_value: PaddingPrimitive) -> PaddingPrimitive {
        let &mut Self {
            horizontal: HorizontalPadding { ref mut right, .. },
            ..
        } = self;

        let previous_value = *right;

        *right = target_value;

        previous_value
    }

    /// Replace the upper padding amount, yielding the previous value for restoration.
    #[inline]
    pub const fn top_replace(&mut self, target_value: PaddingPrimitive) -> PaddingPrimitive {
        let &mut Self {
            vertical: VerticalPadding { ref mut top, .. },
            ..
        } = self;

        let previous_value = *top;

        *top = target_value;

        previous_value
    }

    /// Replace the lower padding amount, yielding the previous value for restoration.
    #[inline]
    pub const fn bottom_replace(&mut self, target_value: PaddingPrimitive) -> PaddingPrimitive {
        let &mut Self {
            vertical: VerticalPadding { ref mut bottom, .. },
            ..
        } = self;

        let previous_value = *bottom;

        *bottom = target_value;

        previous_value
    }

    /// Leading and trailing counts used together to determine extra display width.
    #[inline]
    pub const fn horizontal(&self) -> &HorizontalPadding {
        let &Self { ref horizontal, .. } = self;

        horizontal
    }

    /// Upper and lower counts, kept separate from horizontal space counts.
    #[inline]
    pub const fn vertical(&self) -> &VerticalPadding {
        let &Self { ref vertical, .. } = self;

        vertical
    }

    /// Adjust both horizontal margins without changing vertical padding.
    #[inline]
    pub const fn horizontal_mut(&mut self) -> &mut HorizontalPadding {
        let &mut Self { ref mut horizontal, .. } = self;

        horizontal
    }

    /// Adjust both vertical margins without changing horizontal space counts.
    #[inline]
    pub const fn vertical_mut(&mut self) -> &mut VerticalPadding {
        let &mut Self { ref mut vertical, .. } = self;

        vertical
    }
}
