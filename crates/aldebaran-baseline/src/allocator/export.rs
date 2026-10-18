//! Allocation backend exports from `allocator_api2`.
//!
//! The crate provides `Box` and `Vec` aliases whose default allocator is
//! [`Heap`](crate::allocator::heap::Heap).

#[doc(inline)]
pub use allocator_api2::alloc;

#[doc(inline)]
pub use allocator_api2::collections;

#[doc(inline)]
pub use allocator_api2::SliceExt;

#[doc(inline)]
pub use allocator_api2::unsize_box;

pub mod boxed {
    use allocator_api2::boxed::Box as AllocBox;

    use crate::allocator::heap::Heap;

    /// A type alias to `allocator_api2::boxed::Box` with [`Heap`] as its
    /// default allocator.
    pub type Box<T, A = Heap> = AllocBox<T, A>;
}

pub mod vec {
    use allocator_api2::vec::Vec as AllocVec;

    use crate::allocator::heap::Heap;

    /// A type alias to `allocator_api2::vec::Vec` with [`Heap`] as its default
    /// allocator.
    pub type Vec<T, A = Heap> = AllocVec<T, A>;
}
