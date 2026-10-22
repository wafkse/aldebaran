//! Type-level tuple append and prepend operations.
//!
//! [`Appendage`] grows tuples while preserving their component types, and
//! [`Uniform`] converts homogeneous tuples into fixed-size arrays.

mod appendage;
mod uniform;

pub use self::{appendage::Appendage, uniform::Uniform};
