#![doc = include_str!("../README.md")]
#![cfg_attr(not(test), no_std)]

extern crate alloc;

pub mod zeroable;

pub mod array;

pub mod map;

pub mod set;

pub mod ident;

#[doc(hidden)]
pub mod reexport {
    //! Macro support resolved through this crate, so callers need no direct
    //! dependency on the token-generation facility.

    pub use tokel::stream;
}

pub mod prelude {
    //! The `aldebaran-id` prelude re-exports the crate's commonly used identity
    //! contracts, nonzero representations, and collections.

    pub use crate::Id;

    pub use crate::ident::{Id, TaggedId};

    pub use crate::zeroable::{NonZeroable, NonZeroed};

    pub use crate::array::IdArray;

    pub use crate::map::IdMap;

    pub use crate::set::IdSet;
}
