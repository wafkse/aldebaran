//! Component traversal with source-coordinate offsets.
//!
//! [`SourceIter`] preserves the source's component type. Its
//! [`enumerate`](SourceIter::enumerate) adaptor advances by each component's
//! width, keeping yielded offsets aligned with spans even when component count
//! differs from source length.

use crate::component::Component;
use crate::source::Source;

/// A source that can yield its components in source order.
///
/// Iteration must visit every source component exactly once, with the sum of
/// its [`Component::size`] values equal to the source footprint length. This
/// keeps traversal consistent with the coordinate extent used by source spans.
pub trait SourceIter<'source>: Source<'source> {
    /// Traversal preserving component identity and source order.
    type Iterator<'borrow>: Iterator<Item = Self::Component>
    where
        Self: 'borrow;

    /// Traverse source components without substituting coordinate units for values.
    fn iter(&self) -> Self::Iterator<'_>;

    /// Pair components with offsets in source units, starting at zero.
    /// String offsets count UTF-8 bytes rather than Unicode scalar values.
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

/// Iterator adaptor that pairs source components with their source offsets.
///
/// The first offset is zero. Each subsequent offset advances by the preceding
/// component's nonzero width, so Unicode scalar iteration can still produce
/// byte coordinates. Width sums must fit in `usize`.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
// NOTE(invariant): NextOffset = Sum(ComponentWidths(Yielded)) <= UsizeMax
pub struct EnumeratedComponent<I, C>
where
    I: Iterator<Item = C>,
    C: Component,
{
    /// The remaining source components.
    iter: I,

    /// Offset of the next component relative to the supplied iterator's origin.
    target_index: usize,
}

impl<I, C> EnumeratedComponent<I, C>
where
    I: Iterator<Item = C>,
    C: Component,
{
    /// Assign zero-based source offsets to the supplied component sequence.
    /// The sum of component widths must fit in `usize`.
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
