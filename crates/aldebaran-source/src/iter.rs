//! Component iteration over source representations.
//!
//! See [`SourceIter`] and [`EnumeratedComponent`] for further information.

use crate::component::Component;
use crate::source::Source;

/// A trait for sources that can be arbitrarily iterated over.
///
/// This is an **extension trait**, and thus only builds functionality on top of
/// [`Source`].
///
/// Iteration must visit every source component exactly once, with the sum of
/// its [`Component::size`] values equal to the source footprint length. This
/// keeps traversal consistent with the coordinate extent used by source spans.
pub trait SourceIter<'source>: Source<'source> {
    /// Iterator produced for one borrow of this source.
    type Iterator<'borrow>: Iterator<Item = Self::Component>
    where
        Self: 'borrow;

    /// Retrieve an iterator over the source components.
    fn iter(&self) -> Self::Iterator<'_>;

    /// Iterate over the components, enumerated in this source plane.
    ///
    /// Offsets advance by component width rather than iterator count, so UTF-8
    /// strings retain byte coordinates even though iteration yields `char`s.
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

/// Iterator adaptor that pairs source components with source offsets.
///
/// The running index advances by each component's nonzero width rather than by
/// iterator count. This keeps byte coordinates correct for variable-width source
/// components such as Unicode scalar values in UTF-8 strings. The sum of widths
/// must fit in `usize`.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
// NOTE(invariant): NextOffset = Sum(ComponentWidths(Yielded)) <= UsizeMax
pub struct EnumeratedComponent<I, C>
where
    I: Iterator<Item = C>,
    C: Component,
{
    /// The remaining source components.
    iter: I,

    /// The offset of the next component relative to the supplied iterator's origin.
    target_index: usize,
}

impl<I, C> EnumeratedComponent<I, C>
where
    I: Iterator<Item = C>,
    C: Component,
{
    /// Create a new [`EnumeratedComponent`] from an iterator of components.
    /// The offsets start at zero, and the sum of component widths must fit in `usize`.
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
            ref mut target_index,
        } = self;

        let target_value = iter.next()?;

        let previous_index = *target_index;

        *target_index += target_value.size().get();

        Some((previous_index, target_value))
    }
}
