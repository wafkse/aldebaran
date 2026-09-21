//! Arbitrary object identification.
//!
//! This crate provides a way to identify objects in a way that is unique and
//! consistent.
//!
//! The central trait of this crate is [`Id`], which is a trait that provides a
//! way to identify objects in a generic manner.
#![cfg_attr(not(test), no_std)]

pub mod zeroable;

pub mod array;

pub mod map;

pub mod set;

pub mod ident;

#[doc(hidden)]
pub mod reexport {
    pub use tokel::stream;
}

pub mod prelude {
    //! A prelude for the `aldebaran-id` crate.
    //!
    //! This simply re-exports the most commonly used items in the
    //! `aldebaran-id` crate.

    pub use crate::Id;

    pub use crate::ident::{Id, TaggedId};

    pub use crate::zeroable::{NonZeroable, NonZeroed};

    pub use crate::array::IdArray;

    pub use crate::map::IdMap;

    pub use crate::set::IdSet;
}
