#![no_std]
#![doc = include_str!("../README.md")]

extern crate alloc;

#[cfg(test)]
extern crate std;

pub mod borrow;

pub mod owned;

#[cfg(feature = "source-storage")]
pub mod storage;

pub mod prelude {
    //! The `aldebaran-interner` prelude re-exports the crate's commonly used
    //! storage contracts and, when enabled, source storage implementations.

    pub use crate::borrow::BorrowInterner;

    pub use crate::owned::OwnedInterner;

    #[cfg(feature = "source-storage")]
    pub use crate::storage::{OwnedStorage, Storage, StorageId};
}
