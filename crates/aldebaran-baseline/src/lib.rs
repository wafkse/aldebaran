#![no_std]
#![doc = include_str!("../README.md")]

#[cfg(any(feature = "alloc", test))]
extern crate alloc;

pub mod maybe_spanned;
pub mod select;
pub mod span;
pub mod spanned;
pub mod static_vec;
pub mod tuple;

#[cfg(feature = "alloc")]
pub mod nonempty;

#[cfg(feature = "allocator")]
pub mod allocator;

#[cfg(test)]
mod tests;

pub mod prelude {
    //! The baseline crate prelude re-exports its commonly used coordinate, selection,
    //! and collection primitives.

    pub use crate::maybe_spanned::MaybeSpanned;
    pub use crate::select::{Either, EitherOrBoth};
    pub use crate::span::Span;
    pub use crate::spanned::Spanned;
    pub use crate::static_vec::StaticVec;
    pub use crate::tuple::{Appendage, Uniform};

    #[cfg(feature = "alloc")]
    pub use crate::nonempty::{OneOrMore, RefOneOrMore};
}
