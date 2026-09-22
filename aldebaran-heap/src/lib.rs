#![doc = include_str!("../README.md")]
#![no_std]

pub mod heap;

pub mod profile;

/// Re-export the stable `allocator_api2` allocation backend.
mod export;

pub use export::*;

pub mod prelude {
    //! A prelude for the `aldebaran-heap` crate.
    //!
    //! This simply re-exports the most commonly used items in the
    //! `aldebaran-heap` crate.

    pub use crate::alloc::Allocator;

    pub use crate::heap::Heap;
}
