//! Minimal token stream primitives for hand-written parsers.
//!
//! The crate defines incremental token consumption and fixed-capability
//! lookahead without owning parser state or syntax trees. Stream wrappers add
//! filtering and borrowed or static backing while preserving one token API.
#![cfg_attr(not(test), no_std)]

pub mod stream;

pub mod prelude {
    //! A prelude for the `aldebaran-grammar` crate.
    //!
    //! This simply re-exports the most commonly used items in the
    //! `aldebaran-grammar` crate.

    pub use crate::stream::{TokenStream, lookahead::Lookahead, skipping::Skipping, slice::Slice, storage::Store};
}
