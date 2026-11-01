//! Semantic classification layered on top of logical assertions.
//!
//! See [`Choose`] for further information.

use crate::{assert::Assert, input::Input};

/// A classifier that produces a semantic unit for accepted input.
///
/// Classification refines [`Assert`]. Implementations must return `Some` exactly
/// when the corresponding assertion accepts the same input.
pub trait Choose<I>: Assert<I>
where
    I: Input,
{
    /// Semantic value produced for one accepted input.
    type Unit;

    /// Classify one input accepted by this assertion.
    fn choose(&self, input: I) -> Option<Self::Unit>;
}
