//! Heap allocator wrappers and allocator-backed collection aliases.
//!
//! The module defines [`Heap`], which couples an allocator backend with optional
//! allocation profiling while remaining compatible with `allocator-api2`.

use crate::alloc::{AllocError, Allocator, Global};
use crate::profile::{Profile, Profiled, Profiler};

use core::alloc::Layout;
use core::ptr::NonNull;
use core::{fmt, hash};

/// The default, heap-backed general-purpose allocator.
///
/// Note that this is simply a wrapper around the [`Global`] allocator, in
/// conjunction with the [`Profile`] system.
#[derive(Clone, Copy)]
pub struct Heap<A: Allocator = Global, const P: Profiled = { Profile::none() }> {
    profiler: Option<&'static dyn Profiler>,
    alloc: A,
}

impl<A: Allocator, const P: Profiled> Eq for Heap<A, P> where A: Eq {}

impl<A: Allocator, const P: Profiled> PartialEq for Heap<A, P>
where
    A: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let Self { alloc, .. } = self;

        let Self { alloc: other_alloc, .. } = other;

        alloc == other_alloc
    }
}

impl<A: Allocator, const P: Profiled> hash::Hash for Heap<A, P>
where
    A: hash::Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { alloc, .. } = self;

        alloc.hash(state);
    }
}

impl<A: fmt::Debug, const P: Profiled> fmt::Debug for Heap<A, P>
where
    A: Allocator,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let Self { profiler, .. } = self;

        write!(f, "<heap allocator for")?;

        if let Some(name) = profiler.and_then(|profiler| profiler.name(Profile::is(P))) {
            write!(f, " {}", name)?;
        } else {
            write!(f, " (unnamed) {:2X}", P)?;
        }

        write!(f, ">")
    }
}

impl<const P: Profiled> Heap<Global, P> {
    /// The assigned default value for the [`Heap`] type.
    ///
    /// Note that this is the same as the [`Global`] allocator, with no
    /// profiling capabilities.
    pub const DEFAULT: Self = Self {
        profiler: None,
        alloc: Global,
    };
}

unsafe impl<A: Allocator, const P: Profiled> Allocator for Heap<A, P> {
    #[inline]
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let Self { alloc, .. } = self;

        alloc.allocate(layout)
    }

    #[inline]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        let Self { alloc, .. } = self;

        // SAFETY: upheld by upstream caller
        unsafe {
            alloc.deallocate(ptr, layout);
        }
    }
}

impl<A: Allocator, const P: Profiled> Default for Heap<A, P>
where
    A: Default,
{
    fn default() -> Self {
        Self {
            profiler: None,
            alloc: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{alloc::Global, boxed::Box, heap::Heap, profile::Profile};

    #[test]
    fn default_heap_supports_allocator_backed_box_lifecycle() {
        type DefaultHeap = Heap<Global, { Profile::none() }>;

        let value = Box::new_in(42_u32, DefaultHeap::DEFAULT);

        assert_eq!(*value, 42);
    }
}
