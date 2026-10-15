#![no_std]
#![doc = include_str!("../README.md")]

pub mod visual;

pub mod prelude {
    //! A prelude for the `aldebaran-visualize` crate.
    //!
    //! This simply re-exports the most commonly used items in the
    //! `aldebaran-visualize` crate.

    pub use crate::visual::Visualize;
}
