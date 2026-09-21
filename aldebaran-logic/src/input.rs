//! Value requirements shared by assertion and classification inputs.
//!
//! [`Input`] marks values that can be copied into multiple predicate branches and
//! compared deterministically. The blanket implementation keeps the abstraction
//! zero-cost while giving assertion traits one explicit input contract.

/// An input to an [`Assert`].
///
/// [`Assert`]: crate::assert::Assert
pub trait Input: Clone + Copy + Eq + PartialEq + Ord + PartialOrd + Sized {}

impl<T> Input for T where T: Clone + Copy + Eq + PartialEq + Ord + PartialOrd + Sized {}
