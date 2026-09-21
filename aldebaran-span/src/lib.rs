#![no_std]
#![forbid(
    missing_docs,
    missing_debug_implementations,
    clippy::missing_panics_doc,
    clippy::missing_safety_doc,
    clippy::missing_transmute_annotations,
    clippy::unwrap_used
)]
#![doc = include_str!("../README.md")]

pub mod span;

pub mod spanned;

pub mod maybe_spanned;

#[cfg(test)]
mod tests;

pub mod prelude {
    //! A prelude for the `aldebaran-span` crate.
    //!
    //! This simply re-exports the most commonly used items in the
    //! `aldebaran-span` crate.

    pub use crate::span::Span;

    pub use crate::spanned::Spanned;

    pub use crate::maybe_spanned::MaybeSpanned;
}
