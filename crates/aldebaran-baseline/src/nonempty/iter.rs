use core::{
    iter::{self, FusedIterator},
    slice,
};

use super::{OneOrMore, reference::RefOneOrMore};

/// An iterator that is guaranteed to yield at least one element.
///
/// Procedent from the [`RefOneOrMore`] type.
#[derive(Debug)]
pub struct OnceOrMore<'borrow, T> {
    /// Remaining shared references, with the required first element before the tail.
    inner: iter::Chain<iter::Once<&'borrow T>, slice::Iter<'borrow, T>>,
}

impl<'borrow, T> Iterator for OnceOrMore<'borrow, T> {
    type Item = &'borrow T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let &mut Self { ref mut inner } = self;

        inner.next()
    }
}

impl<'borrow, T> FusedIterator for OnceOrMore<'borrow, T> {}

impl<'borrow, T> OnceOrMore<'borrow, T> {
    /// Create a new [`OnceOrMore`] sourcing from a reference-based
    /// [`RefOneOrMore`].
    #[inline]
    pub fn from_ref(value: RefOneOrMore<'borrow, T>) -> Self {
        let (a0, a_n) = (value.first(), value.rest());

        Self {
            inner: iter::once(a0).chain(a_n.iter()),
        }
    }
}

/// An iterator that is guaranteed to yield at least one mutable element
/// reference.
#[derive(Debug)]
pub struct OnceOrMoreMut<'borrow, T> {
    /// Remaining disjoint mutable references in first-element and tail order.
    inner: iter::Chain<iter::Once<&'borrow mut T>, slice::IterMut<'borrow, T>>,
}

impl<'borrow, T> OnceOrMoreMut<'borrow, T> {
    /// Borrow the required first element and tail exclusively for traversal.
    #[inline]
    pub fn from_mut_ref(value: &'borrow mut OneOrMore<T>) -> Self {
        let (a0, a_n) = value.tuple_mut();

        Self {
            inner: iter::once(a0).chain(a_n.iter_mut()),
        }
    }
}

impl<'borrow, T> Iterator for OnceOrMoreMut<'borrow, T> {
    type Item = &'borrow mut T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let &mut Self { ref mut inner } = self;

        inner.next()
    }
}

impl<'borrow, T> FusedIterator for OnceOrMoreMut<'borrow, T> {}
