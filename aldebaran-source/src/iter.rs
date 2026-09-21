//! Component iteration over source representations.
//!
//! [`SourceIter`] exposes the source's natural component iterator without erasing
//! component width. [`EnumeratedComponent`] accumulates those widths so each
//! yielded component is paired with its absolute boundary in the source plane.

use crate::component::Component;
use crate::source::Source;

/// A trait for sources that can be arbitrarily iterated over.
///
/// This is an **extension trait**, and thus only builds functionality on top of
/// [`Source`].
pub trait SourceIter<'source>: Source<'source> {
    /// Iterator produced for one borrow of this source.
    type Iterator<'borrow>: Iterator<Item = Self::Component>
    where
        Self: 'borrow;

    /// Retrieve an iterator over the source components.
    fn iter(&self) -> Self::Iterator<'_>;

    /// Iterate over the components, enumerated in this source plane.
    #[inline]
    fn enumerate(&self) -> EnumeratedComponent<Self::Iterator<'_>, Self::Component> {
        EnumeratedComponent::enumerate(self.iter())
    }
}

/// Blanket implementation for references to iterable sources.
impl<'source, S> SourceIter<'source> for &S
where
    S: SourceIter<'source> + ?Sized,
    Self: Source<'source, Component = S::Component>,
{
    type Iterator<'borrow>
        = S::Iterator<'borrow>
    where
        Self: 'borrow;

    #[inline]
    fn iter(&self) -> Self::Iterator<'_> {
        <S as SourceIter<'source>>::iter(self)
    }
}

/// Iterator adaptor that pairs source components with absolute source offsets.
///
/// The running index advances by each component's nonzero width rather than by
/// iterator count. This keeps byte coordinates correct for variable-width source
/// components such as Unicode scalar values in UTF-8 strings.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct EnumeratedComponent<I, C>
where
    I: Iterator<Item = C>,
    C: Component,
{
    iter: I,
    target_index: usize,
}

impl<I, C> EnumeratedComponent<I, C>
where
    I: Iterator<Item = C>,
    C: Component,
{
    /// Create a new [`EnumeratedComponent`] from an iterator of components.
    #[inline]
    pub const fn enumerate(iter: I) -> Self {
        Self { iter, target_index: 0 }
    }
}

impl<I, C> Iterator for EnumeratedComponent<I, C>
where
    I: Iterator<Item = C>,
    C: Component,
{
    type Item = (usize, C);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let &mut Self {
            ref mut iter,
            target_index: previous_index,
        } = self;

        let target_value = iter.next()?;

        let target_index = previous_index + target_value.size().get();

        self.target_index = target_index;

        Some((previous_index, target_value))
    }
}
