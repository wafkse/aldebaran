#![doc = include_str!("../README.md")]
#![no_std]
#![cfg_attr(feature = "nightly", feature(allocator_api))]

pub mod heap;

pub mod profile;

/// Re-export module for either the `allocator_api2`, or the `alloc` crate.
mod export;

pub use export::*;

/// A prelude module for convenience. Re-exports commonly used items.
pub mod prelude {
    //! A prelude for the `aldebaran-heap` crate.
    //!
    //! This simply re-exports the most commonly used items in the
    //! `aldebaran-heap` crate.

    pub use crate::alloc::Allocator;

    pub use crate::heap::Heap;
}
