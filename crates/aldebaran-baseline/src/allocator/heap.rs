//! Allocator adapters with profile labels for diagnostic output.
//!
//! [`Heap`] delegates memory operations to the selected [`Allocator`]. Its
//! profile parameter distinguishes debug labels without changing allocation
//! behavior or ownership of the underlying allocations.

use crate::allocator::{
    export::alloc::{AllocError, Allocator, Global},
    profile::{Profile, Profiled, Profiler},
};

use core::alloc::Layout;
use core::ptr::NonNull;
use core::{fmt, hash};

/// An allocator with a profile label used only for debug output.
///
/// Memory operations retain `A`'s allocation contract. Equality and hashing use
/// the underlying allocator independently of its optional debug-name provider.
#[derive(Clone, Copy)]
// NOTE(invariant): None.
pub struct Heap<A: Allocator = Global, const P: Profiled = { Profile::none() }> {
    /// Optional provider for the allocator's debug name.
    profiler: Option<&'static dyn Profiler>,

    /// Allocator that performs the underlying memory operations.
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
    #[inline]
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
        let Self { alloc, .. } = self;

        alloc.hash(state);
    }
}

impl<A: fmt::Debug, const P: Profiled> fmt::Debug for Heap<A, P>
where
    A: Allocator,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
    /// A heap backed by [`Global`].
    pub const DEFAULT: Self = Self {
        profiler: None,
        alloc: Global,
    };
}

// SAFETY: Allocation and deallocation are forwarded unchanged to `A`, which
// implements the `Allocator` contract. Heap stores no additional allocation state.
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
    #[inline]
    fn default() -> Self {
        Self {
            profiler: None,
            alloc: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::allocator::{
        export::{alloc::Global, boxed::Box},
        heap::Heap,
        profile::Profile,
    };

    #[test]
    fn default_heap_supports_allocator_backed_box_lifecycle() {
        type DefaultHeap = Heap<Global, { Profile::none() }>;

        let value = Box::new_in(42_u32, DefaultHeap::DEFAULT);

        assert_eq!(*value, 42);
    }
}
