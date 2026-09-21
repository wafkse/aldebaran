#![no_std]
#![doc = include_str!("../README.md")]

pub mod combine;
pub mod print;

pub mod prelude {
    //! A prelude for the `aldebaran-print` crate.
    //!
    //! This simply re-exports the most commonly used items in the
    //! `aldebaran-print` crate.

    pub use crate::combine::Combine;
    pub use crate::print::Print;
}
