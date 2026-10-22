//! The baseline crate prelude re-exports its commonly used coordinate, selection,
//! and collection primitives.

pub use crate::select::{Either, EitherOrBoth};
pub use crate::span::{MaybeSpanned, Span, Spanned};
pub use crate::tuple::{Appendage, Uniform};

#[cfg(feature = "alloc")]
pub use crate::nonempty::{Nonempty, RefNonempty};
