//! Classification of inputs accepted by an assertion. [`Choose`] adds a typed
//! result whose presence must agree with the predicate's acceptance decision.

use crate::{assert::Assert, input::Input};

/// Adds a classification result to an [`Assert`] predicate.
///
/// Implementers must make [`choose`](Self::choose) return `Some` exactly when
/// [`Assert::assert`] accepts the same input.
pub trait Choose<I>: Assert<I>
where
    I: Input,
{
    /// Classification identifying an accepted form, retained when classifiers are composed.
    type Unit;

    /// Return this input's classification, or `None` when the assertion rejects it.
    fn choose(&self, input: I) -> Option<Self::Unit>;
}
