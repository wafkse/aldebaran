#![no_std]
#![doc = include_str!("../../../docs/part/COMPONENT.md")]
#![doc = include_str!("../README.md")]
#![doc = include_str!("../../../docs/part/FACADE.md")]
#![doc = include_str!("../../../docs/part/LICENSE.md")]

#[cfg(any(feature = "alloc", test))]
extern crate alloc;

pub mod select;
pub mod span;
pub mod tuple;

#[cfg(feature = "alloc")]
pub mod nonempty;

pub mod prelude;
