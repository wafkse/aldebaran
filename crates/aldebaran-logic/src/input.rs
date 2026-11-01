//! Values that predicates can compare and reuse across a composition.

/// An input to an [`Assert`].
///
/// Inputs can be supplied unchanged to several assertion operands. Any type
/// satisfying the comparison and copying bounds qualifies automatically.
///
/// [`Assert`]: crate::assert::Assert
pub trait Input: Clone + Copy + Eq + PartialEq + Ord + PartialOrd + Sized {}

impl<T> Input for T where T: Clone + Copy + Eq + PartialEq + Ord + PartialOrd + Sized {}
