//! Allocation backend exports from `allocator_api2`.
//!
//! The crate provides `Box` and `Vec` aliases whose default allocator is
//! [`Heap`](crate::heap::Heap).

#[doc(inline)]
pub use allocator_api2::alloc;

#[doc(inline)]
pub use allocator_api2::collections;

#[doc(inline)]
pub use allocator_api2::SliceExt;

#[doc(inline)]
pub use allocator_api2::unsize_box;

/// Boxed allocation support with [`crate::heap::Heap`] as the default
/// allocator.
pub mod boxed {
    use crate::heap::Heap;

    /// A type alias to `allocator_api2::boxed::Box` with [`Heap`] as its
    /// default allocator.
    pub type Box<T, A = Heap> = allocator_api2::boxed::Box<T, A>;
}

/// Vector allocation support with [`crate::heap::Heap`] as the default
/// allocator.
pub mod vec {
    use crate::heap::Heap;

    /// A type alias to `allocator_api2::vec::Vec` with [`Heap`] as its default
    /// allocator.
    pub type Vec<T, A = Heap> = allocator_api2::vec::Vec<T, A>;
}
