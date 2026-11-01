//! Type-level tuple append and prepend operations.
//!
//! See [`Appendage`] and [`Uniform`] for further information.

mod appendage;
mod uniform;

pub use self::{appendage::Appendage, uniform::Uniform};
