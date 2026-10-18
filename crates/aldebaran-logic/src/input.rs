//! Values that predicates can compare and reuse across a composition.

/// An input that can be supplied unchanged to several assertion operands.
/// All types satisfying the comparison and copying bounds qualify automatically.
pub trait Input: Clone + Copy + Eq + PartialEq + Ord + PartialOrd + Sized {}

impl<T> Input for T where T: Clone + Copy + Eq + PartialEq + Ord + PartialOrd + Sized {}
