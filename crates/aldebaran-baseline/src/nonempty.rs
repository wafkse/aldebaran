//! Collections with a required first value and an optional tail.
//!
//! [`Nonempty`] owns its values, while [`RefNonempty`] shares a first
//! value and a slice without allocating. Both traverse the first value before
//! the tail, so diagnostic collections can retain a primary value separately
//! from related values.

mod owned;
mod reference;

pub use self::{owned::Nonempty, reference::RefNonempty};
