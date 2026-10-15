#![doc = include_str!("../README.md")]
#![no_std]

extern crate alloc;

pub mod select;

pub mod collect;

pub mod appendage;

pub mod prelude {
    //! A prelude for the `aldebaran-dsa` crate.
    //!
    //! This simply re-exports the most commonly used items in the
    //! `aldebaran-dsa` crate.

    pub use crate::collect::*;

    pub use crate::appendage::Appendage;
}
