#![doc = include_str!("../README.md")]
#![cfg_attr(not(test), no_std)]

pub mod stream;

pub mod prelude {
    //! The `aldebaran-grammar` prelude re-exports the crate's commonly used
    //! stream contracts and adapters.

    pub use crate::stream::{TokenStream, lookahead::Lookahead, skipping::Skipping, slice::Slice, storage::Store};
}
