//! Owned collections that guarantee at least one element.
//!
//! See [`Nonempty`] and [`RefNonempty`] for further information.

mod owned;
mod reference;

pub use self::{owned::Nonempty, reference::RefNonempty};
