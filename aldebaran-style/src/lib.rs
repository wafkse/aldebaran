#![no_std]
#![forbid(unsafe_code, rustdoc::all, clippy::panic, clippy::unwrap_used)]
//! Facilities for generic text styling.
//!
//! This crate provides facilities to approach text styling in a generic, and
//! extensible manner.
//!
//! The main trait of this crate is the [`Stylus`](stylus::Stylus) trait, which
//! allows for the application of styles to a writer.

mod stylus;

pub mod prelude {
    //! A prelude for the `aldebaran-style` crate.
    //!
    //! This simply re-exports the most commonly used items in the
    //! `aldebaran-style` crate.

    pub use crate::stylus::Stylus;
}
