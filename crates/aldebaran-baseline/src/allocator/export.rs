//! Exports allocator interfaces and collections that use [`Heap`](crate::allocator::heap::Heap) by default.

#[doc(inline)]
pub use allocator_api2::alloc;

#[doc(inline)]
pub use allocator_api2::collections;

#[doc(inline)]
pub use allocator_api2::SliceExt;

#[doc(inline)]
pub use allocator_api2::unsize_box;

/// Allocator-aware box aliases defaulting to [`crate::allocator::heap::Heap`].
pub mod boxed {
    use allocator_api2::boxed::Box as AllocBox;

    use crate::allocator::heap::Heap;

    /// An allocator-aware box that uses [`Heap`] by default.
    pub type Box<T, A = Heap> = AllocBox<T, A>;
}

/// Allocator-aware vector aliases defaulting to [`crate::allocator::heap::Heap`].
pub mod vec {
    use allocator_api2::vec::Vec as AllocVec;

    use crate::allocator::heap::Heap;

    /// An allocator-aware vector that uses [`Heap`] by default.
    pub type Vec<T, A = Heap> = AllocVec<T, A>;
}
