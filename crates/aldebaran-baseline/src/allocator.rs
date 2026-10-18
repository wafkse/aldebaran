//! Allocator selection for allocation-backed collections.
//!
//! [`heap::Heap`] adapts an allocator with a profile label for debug output.
//! [`profile`] defines those labels, and [`export`] supplies collection aliases
//! that use the adapter by default. This module requires the `allocator` feature.

pub mod export;
pub mod heap;
pub mod profile;

pub use self::{
    heap::Heap,
    profile::{Profile, Profiled, Profiler},
};
