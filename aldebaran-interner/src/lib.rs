#![no_std]
#![forbid(unsafe_attr_outside_unsafe, clippy::all, clippy::redundant_allocation, clippy::missing_safety_doc)]
//! Interning facilities for stable value identity.
//!
//! Borrowed and owned interners provide distinct storage models while sharing
//! the same role of mapping repeated values to compact identities. Callers pick
//! the ownership model that matches the lifetime of their source data.

extern crate alloc;

pub mod borrow;

pub mod owned;

pub mod prelude {
    //! A prelude for the `aldebaran-interner` crate.
    //!
    //! This simply re-exports the most commonly used items in the
    //! `aldebaran-interner` crate.

    pub use crate::borrow::BorrowInterner;

    pub use crate::owned::OwnedInterner;
}
