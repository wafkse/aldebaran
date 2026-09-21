//! Fixed-capacity vector storage without heap allocation.
//!
//! [`StaticVec`] keeps up to `N` initialized elements inside an inline
//! `MaybeUninit` array and exposes the initialized prefix as a slice. Push and
//! pop operations maintain the initialized-length invariant without reallocating.

use core::{
    fmt,
    mem::{self, MaybeUninit},
    ops::{Deref, DerefMut},
};

/// Fixed-capacity vector backed by inline uninitialized storage.
///
/// The type tracks exactly how many array slots contain initialized values and
/// exposes only that prefix through slice operations. Capacity never changes and
/// mutation fails explicitly when no unused slot remains.
///
/// This both [`Deref`]s and [`DerefMut`]s to a slice of `T`, so it can be used
/// with the usual, well-known slice methods.
pub struct StaticVec<T, const N: usize> {
    /// The backing storage array for this [`StaticVec`].
    ///
    /// This is the first-field of the struct to aid in pointer casting, since
    /// this is a struct very likely to be dumped to the stack.
    storage_buffer: [MaybeUninit<T>; N],

    /// The amount of elements currently stored in this [`StaticVec`].
    // NOTE(invariant): `storage_len` never exceeds `N`, and exactly the first `storage_len` slots are initialized.
    storage_len: usize,
}

impl<T, const N: usize> StaticVec<T, N> {
    /// The total amount of elements this [`StaticVec`] can store at maximum.
    pub const STORAGE_SIZE: usize = N;
}

impl<T, const N: usize> StaticVec<T, N> {
    /// An empty [`StaticVec`] that is of capacity `N`.
    #[inline]
    pub const fn empty() -> Self {
        Self {
            // FIXME: move to `MaybeUninit::uninit_array()` once it is stable.
            storage_buffer: [const { MaybeUninit::uninit() }; N],
            storage_len: 0,
        }
    }
}

impl<T, const N: usize> StaticVec<T, N> {
    /// Attempt to push an element `T` into this [`StaticVec`].
    ///
    /// Fails if the [`StaticVec`] is already at capacity.
    #[inline]
    pub const fn push(&mut self, value: T) -> Result<(), T> {
        let &mut Self {
            ref mut storage_buffer,
            ref mut storage_len,
        } = self;

        if *storage_len == N {
            return Err(value);
        }

        // FIXME: use `get_unchecked_mut` once it can be used in const fn
        let target_slot = &mut storage_buffer[*storage_len];

        // SAFETY: this is a write to uninitialized memory, which is safe.
        unsafe {
            target_slot.as_mut_ptr().write(value);
        }

        *storage_len += 1;

        Ok(())
    }

    /// Pop a value from this [`StaticVec`].
    #[inline]
    pub const fn pop(&mut self) -> Option<T> {
        let &mut Self {
            ref mut storage_buffer,
            ref mut storage_len,
        } = self;

        if *storage_len == 0 {
            return None;
        }

        *storage_len -= 1;

        // FIXME: use `get_unchecked_mut` once it can be used in const fn
        let target_slot = &mut storage_buffer[*storage_len];

        let target_value = mem::replace(target_slot, MaybeUninit::uninit());

        // SAFETY: `target_value` is initialized, since it was taken from a
        // `MaybeUninit` slot that had a value written to it.
        let target_value = unsafe { target_value.assume_init() };

        Some(target_value)
    }
}

impl<T, const N: usize> StaticVec<T, N> {
    /// Determine the amount of elements stored in this [`StaticVec`].
    #[inline]
    pub const fn len(&self) -> usize {
        let &Self { storage_len, .. } = self;

        storage_len
    }

    /// Determine the capacity of this [`StaticVec`].
    #[inline]
    pub const fn capacity(&self) -> usize {
        let &Self { storage_len, .. } = self;

        // NOTE: avoid overflow checks here, since `storage_len <= N` by
        // definition.
        N.wrapping_sub(storage_len)
    }
}

impl<T, const N: usize> StaticVec<T, N> {
    /// Reinterpret this [`StaticVec`] as an immutable slice of `T`.
    #[inline]
    pub const fn as_slice(&self) -> &[T] {
        let &Self {
            ref storage_buffer,
            storage_len: len,
        } = self;

        let storage_slice = storage_buffer.as_slice();

        let storage_addr = storage_slice.as_ptr().cast::<T>();

        // SAFETY: The pointer is valid, since it was obtained from a valid
        // slice of `MaybeUninit<T>`, which has the same representation as `T`.
        unsafe { core::slice::from_raw_parts(storage_addr, len) }
    }

    /// Reinterpret this [`StaticVec`] as an immutable slice of `T`.
    #[inline]
    pub const fn as_slice_mut(&mut self) -> &mut [T] {
        let &mut Self {
            ref mut storage_buffer,
            storage_len: len,
        } = self;

        // FIXME: use `as_mut_slice` once stable in const fn
        let storage_slice = storage_buffer.as_slice();

        let storage_addr = storage_slice.as_ptr().cast::<T>().cast_mut();

        // SAFETY: The pointer is valid, since it was obtained from a valid
        // slice of `MaybeUninit<T>`, which has the same representation as `T`.
        unsafe { core::slice::from_raw_parts_mut(storage_addr, len) }
    }
}

impl<T, const N: usize> Deref for StaticVec<T, N> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T, const N: usize> DerefMut for StaticVec<T, N> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_slice_mut()
    }
}

impl<T, const N: usize> Drop for StaticVec<T, N> {
    #[inline]
    fn drop(&mut self) {
        let &mut Self {
            ref mut storage_buffer,
            storage_len,
        } = self;

        // SAFETY: The elements in the storage buffer are not dropped, since
        // they are `MaybeUninit`. So, we can safely drop the elements
        // in the buffer.
        unsafe {
            storage_buffer
                .iter_mut()
                .take(storage_len)
                .for_each(|v| MaybeUninit::assume_init_drop(v));
        }
    }
}

impl<T: fmt::Debug, const N: usize> fmt::Debug for StaticVec<T, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}
