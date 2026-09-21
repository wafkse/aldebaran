#![no_std]
#![doc = include_str!("../README.md")]
#![forbid(unsafe_code, missing_docs, rustdoc::all, clippy::all, clippy::pedantic)]

pub mod visual;

pub mod prelude {
    //! A prelude for the `aldebaran-visualize` crate.
    //!
    //! This simply re-exports the most commonly used items in the
    //! `aldebaran-visualize` crate.

    pub use crate::visual::Visualize;
}
