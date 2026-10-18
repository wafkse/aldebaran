//! Optional allocation support built on `allocator-api2`.
//!
//! The allocator wrapper forwards memory operations to its backend and carries
//! optional profile metadata. Profile names do not imply allocation tracing.

pub mod export;
pub mod heap;
pub mod profile;

pub use self::{
    heap::Heap,
    profile::{Profile, Profiled, Profiler},
};
