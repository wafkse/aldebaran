//! Borrowed views and iteration for owned nonempty collections.

/// Shared and exclusive traversal over the required first element and tail.
pub mod iter;
pub mod reference;

pub use self::iter::{OnceOrMore, OnceOrMoreMut};
pub use self::reference::RefOneOrMore;

use core::{convert::AsRef, num::NonZero};

use alloc::vec::Vec;

/// An owned sequence with separate access to its first value and tail.
///
/// For a borrowed view, see [`RefOneOrMore`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
// NOTE(invariant): None.
pub struct OneOrMore<T> {
    /// The required first element that makes the collection nonempty.
    a0: T,

    /// Owned elements following the required first element.
    a_n: Vec<T>,
}

impl<T> OneOrMore<T> {
    /// Create a collection containing a reference to the first input value.
    ///
    /// The returned tail starts empty, although its requested capacity comes from
    /// the input tail's length. As a result, the input tail reserves storage without
    /// contributing values to the collection.
    #[inline]
    pub fn references<'a>((a0, a_n): (&'a T, &'a [T])) -> OneOrMore<&'a T> {
        let a_n = Vec::with_capacity(a_n.len());

        OneOrMore { a0, a_n }
    }

    /// Borrow the required first value.
    #[inline]
    pub const fn first(&self) -> &T {
        let &Self { ref a0, .. } = self;

        a0
    }

    /// Mutably borrow the required first value.
    #[inline]
    pub const fn first_mut(&mut self) -> &mut T {
        let &mut Self { ref mut a0, .. } = self;

        a0
    }

    /// Borrow the tail following the required first value.
    #[inline]
    pub const fn rest(&self) -> &Vec<T> {
        let &Self { ref a_n, .. } = self;

        a_n
    }

    /// Mutate the tail without removing the required first value.
    #[inline]
    pub const fn rest_mut(&mut self) -> &mut Vec<T> {
        let &mut Self { ref mut a_n, .. } = self;

        a_n
    }

    /// Borrow the required value and tail together.
    #[inline]
    pub const fn tuple(&self) -> (&T, &Vec<T>) {
        let &Self { ref a0, ref a_n } = self;

        (a0, a_n)
    }

    /// Mutably borrow the required value and tail together.
    #[inline]
    pub fn tuple_mut(&mut self) -> (&mut T, &mut Vec<T>) {
        let &mut Self { ref mut a0, ref mut a_n } = self;

        (a0, a_n)
    }

    /// Index the tail using one-based numbering.
    ///
    /// Because the required first value is excluded, position `1` selects the
    /// first tail value, and positions beyond the tail return [`None`]. To include
    /// the required value in traversal, use [`Self::iter`] instead.
    #[inline]
    pub fn nth(&self, n: NonZero<usize>) -> Option<&T> {
        let &Self { ref a_n, .. } = self;

        let target_index = n.get() - 1; // REMARK: will never underflow, as `NonZero` is always at least 1.

        a_n.get(target_index)
    }

    /// Iterate over the required value followed by the tail.
    #[inline]
    pub fn iter(&self) -> OnceOrMore<'_, T> {
        let &Self { ref a0, ref a_n } = self;

        let a0 = RefOneOrMore { a0, a_n };

        a0.iter()
    }

    /// Mutably iterate over the required value followed by the tail.
    #[inline]
    pub fn iter_mut(&mut self) -> OnceOrMoreMut<'_, T> {
        OnceOrMoreMut::from_mut_ref(self)
    }

    /// Borrow this sequence as a [`RefOneOrMore`] without copying its elements.
    #[inline]
    pub fn as_ref(&self) -> RefOneOrMore<'_, T> {
        let &Self { ref a0, ref a_n } = self;

        RefOneOrMore { a0, a_n }
    }
}

impl<T> OneOrMore<T> {
    /// Create a sequence whose tail is empty.
    #[inline]
    pub const fn single(a0: T) -> Self {
        let a_n = Vec::new();

        Self { a0, a_n }
    }

    /// Clone the first value and tail into owned storage.
    #[inline]
    pub fn from_borrowed<P, U>(a0: P, a_n: U) -> Self
    where
        T: Clone,
        P: AsRef<T>,
        U: AsRef<[T]>,
    {
        let a0 = a0.as_ref().clone();

        // FIXME: make use of allocator_api when it's available, otherwise we
        // just manually implement it. let a_n = a_n.as_ref().
        // to_vec_in(A::default());

        let a_n = {
            let target_slice = a_n.as_ref();

            let mut a_n = Vec::with_capacity(target_slice.len());

            a_n.extend_from_slice(target_slice);

            a_n
        };

        Self { a0, a_n }
    }

    /// Create a sequence from its required value and owned tail.
    #[inline]
    pub fn from_raw_parts((a0, a_n): (T, Vec<T>)) -> Self {
        Self { a0, a_n }
    }

    /// Convert a nonempty vector into a [`OneOrMore`].
    ///
    /// The original first element becomes the required value, so an empty vector
    /// returns [`None`]. Removing that first element moves the original last
    /// element to the front of the tail, which means the input order is not
    /// preserved when the vector has more than one element.
    #[inline]
    pub fn from_vec(mut vec: Vec<T>) -> Option<Self> {
        if vec.is_empty() {
            None
        } else {
            let a0 = vec.swap_remove(0);
            let a_n = vec;

            Some(Self { a0, a_n })
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::{vec, vec::Vec};
    use core::num::NonZero;

    use super::{OneOrMore, RefOneOrMore};

    #[test]
    fn from_vec_rejects_empty() {
        assert_eq!(OneOrMore::<u32>::from_vec(Vec::new()), None);
    }

    #[test]
    fn from_vec_preserves_singleton() {
        let values = OneOrMore::from_vec(vec![7]);

        assert_eq!(values, Some(OneOrMore::single(7)));
    }

    #[test]
    fn from_vec_preserves_swap_remove_order() {
        let values = OneOrMore::from_vec(vec![1, 2, 3, 4]);
        let expected = OneOrMore::from_raw_parts((1, vec![4, 2, 3]));

        assert_eq!(values, Some(expected));
    }

    #[test]
    fn borrowed_view_keeps_the_primary_and_tail_without_copying() {
        let values = [3, 5, 8];
        let view = RefOneOrMore::new(&values[0], &values[1..]);

        assert_eq!(*view.first(), 3);
        assert_eq!(view.rest(), &[5, 8]);
        assert_eq!(view.len(), NonZero::new(3).expect("view has a primary element"));
        assert_eq!(view.iter().copied().collect::<Vec<_>>(), values);
    }

    #[test]
    fn owned_iteration_yields_the_required_primary_value() {
        let values = OneOrMore::single(11);

        assert_eq!(values.iter().copied().collect::<Vec<_>>(), [11]);
        assert_eq!(values.as_ref().iter().copied().collect::<Vec<_>>(), [11]);
    }
}
