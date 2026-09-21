//! Allocation backend exports selected by the heap feature set.
//!
//! Nightly builds forward the standard allocation crate. Stable builds forward
//! `allocator_api2` and provide `Box` and `Vec` aliases whose default allocator
//! is the crate [`Heap`](crate::heap::Heap).

//
extern crate alloc as liballoc;

use allocator_api2 as liballoc2;

#[doc(inline)]
#[cfg(feature = "nightly")]
pub use liballoc::*;

#[doc(inline)]
#[cfg(not(feature = "nightly"))]
pub use liballoc2::*;

/// Boxed allocation support using the selected allocation backend.
///
/// Stable builds default the allocator parameter to [`crate::heap::Heap`]. Nightly builds
/// forward the allocation crate implementation directly.
pub mod boxed {
    use crate::heap::Heap;

    #[cfg(feature = "nightly")]
    use super::liballoc;

    #[cfg(not(feature = "nightly"))]
    use super::liballoc2;

    /// A type alias to [`Box`] that makes use of the [`Heap`] allocator as the
    /// default.
    #[cfg(not(feature = "nightly"))]
    pub type Box<T, A = Heap> = liballoc2::boxed::Box<T, A>;

    #[cfg(feature = "nightly")]
    pub use liballoc::boxed::Box;
}

/// Vector allocation support using the selected allocation backend.
///
/// Stable builds default the allocator parameter to [`crate::heap::Heap`]. Nightly builds
/// forward the allocation crate implementation directly.
pub mod vec {
    use crate::heap::Heap;

    #[cfg(feature = "nightly")]
    use super::liballoc;

    #[cfg(not(feature = "nightly"))]
    use super::liballoc2;

    /// A type alias to [`Vec`] that makes use of the [`Heap`] allocator as the
    /// default.
    #[cfg(not(feature = "nightly"))]
    pub type Vec<T, A = Heap> = liballoc2::vec::Vec<T, A>;

    #[cfg(feature = "nightly")]
    pub use liballoc::vec::Vec;
}
