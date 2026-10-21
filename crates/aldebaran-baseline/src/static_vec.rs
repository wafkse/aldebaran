//! Bounded sequences whose elements stay in inline storage.
//!
//! [`StaticVec`] keeps its maximum size in the type and rejects a push when that
//! storage is full. Its slice views expose only occupied positions, while
//! [`StaticVec::capacity`] reports the remaining space rather than the maximum.

use core::{
    fmt,
    mem::{self, MaybeUninit},
    ops::{Deref, DerefMut},
    slice,
};

/// A growable inline sequence with a compile-time maximum of `N` elements.
///
/// Pushing into a full sequence returns the supplied value unchanged. Slice
/// access and dereferencing cover only occupied positions, and removed values
/// transfer back to the caller rather than remaining owned by the sequence.
// NOTE(invariant): `storage_len` never exceeds `N`, and exactly the first `storage_len` slots are initialized.
pub struct StaticVec<T, const N: usize> {
    /// Inline storage for the sequence's elements.
    storage_buffer: [MaybeUninit<T>; N],

    /// Current number of elements.
    storage_len: usize,
}

impl<T, const N: usize> StaticVec<T, N> {
    /// Maximum number of elements this [`StaticVec`] can store.
    pub const STORAGE_SIZE: usize = N;
}

impl<T, const N: usize> StaticVec<T, N> {
    /// Create an empty vector with capacity `N`.
    #[inline]
    pub const fn empty() -> Self {
        Self {
            storage_buffer: [const { MaybeUninit::uninit() }; N],
            storage_len: 0,
        }
    }
}

impl<T, const N: usize> StaticVec<T, N> {
    /// Append an element, returning it unchanged if the vector is full.
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

    /// Remove and return the last element, or [`None`] when empty.
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
    /// Number of initialized elements.
    #[inline]
    pub const fn len(&self) -> usize {
        let &Self { storage_len, .. } = self;

        storage_len
    }

    /// Whether the vector contains no initialized elements.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Number of additional elements that fit before the vector is full.
    #[inline]
    pub const fn capacity(&self) -> usize {
        let &Self { storage_len, .. } = self;

        // NOTE: avoid overflow checks here, since `storage_len <= N` by
        // definition.
        N.wrapping_sub(storage_len)
    }
}

impl<T, const N: usize> StaticVec<T, N> {
    /// Borrow the initialized prefix as a slice.
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
        unsafe { slice::from_raw_parts(storage_addr, len) }
    }

    /// Mutably borrow the initialized prefix as a slice.
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
        unsafe { slice::from_raw_parts_mut(storage_addr, len) }
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
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

#[cfg(test)]
mod tests;
