#![no_std]
#![doc = include_str!("../README.md")]

#[cfg(any(feature = "alloc", test))]
extern crate alloc;

pub mod select;
pub mod span;
pub mod tuple;

#[cfg(feature = "alloc")]
pub mod nonempty;

pub mod prelude;
