//! Aggregate metadata for source line extents.
//!
//! This module records line-number bounds, byte extents, and observed line-length
//! extrema. The metadata is carried by line handles so renderers can size and
//! locate views without rescanning the complete source.

use core::{num::NonZero, ops::RangeInclusive};

use aldebaran_span::prelude::Span;

use aldebaran_primitive::prelude::Primitive;

/// Trait for recordable primitives.
///
/// A primitive is considered recordable if it can be used to record historical
/// minimum and maximum values.
///
/// In other words, a recordable primitive must be arbitrarily-copyable be
/// comparable with total order.
pub trait Record: Primitive + Copy + PartialOrd + Ord {}

impl<R> Record for R where R: Primitive + Copy + PartialOrd + Ord {}

/// A pair of historical minimum and maximum values for a target,
/// [`recordable`](Record) primitive.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Default)]
pub struct MinMax<T = usize>
where
    T: Record,
{
    /// The minimum value recorded.
    min: T,
    /// The maximum value recorded.
    max: T,
}

impl<T> MinMax<T>
where
    T: Record,
{
    /// The range (absolute difference) between the minimum and maximum values.
    #[inline]
    pub const fn range(&self) -> RangeInclusive<T> {
        let &Self { min, max, .. } = self;

        min..=max
    }
}

impl<T> MinMax<T>
where
    T: Record,
{
    /// Construct a [`MinMax`] pair from a single target value.
    ///
    /// This is as effective as setting the minimum and maximum values to the
    /// same value.
    #[inline]
    pub const fn initial(value: T) -> Self {
        let min = value;
        let max = value;

        Self { min, max }
    }

    /// Create a new [`MinMax`] pair from a target 2-element tuple.
    #[inline]
    pub const fn pair((min, max): (T, T)) -> Self {
        Self { min, max }
    }

    /// Retrieve the minimum value recorded.
    #[inline]
    pub const fn min(&self) -> T {
        let &Self { min, .. } = self;

        min
    }

    /// Retrieve the maximum value recorded.
    #[inline]
    pub const fn max(&self) -> T {
        let &Self { max, .. } = self;

        max
    }

    /// Retrieve an immutable reference to the minimum value recorded.
    #[inline]
    pub const fn min_ref(&self) -> &T {
        let &Self { ref min, .. } = self;

        min
    }

    /// Retrieve an immutable reference to the maximum value recorded.
    #[inline]
    pub const fn max_ref(&self) -> &T {
        let &Self { ref max, .. } = self;

        max
    }

    /// Retrieve an immutable reference to the minimum value recorded.
    #[inline]
    pub const fn min_mut(&mut self) -> &mut T {
        let &mut Self { ref mut min, .. } = self;

        min
    }

    /// Retrieve an immutable reference to the maximum value recorded.
    #[inline]
    pub const fn max_mut(&mut self) -> &mut T {
        let &mut Self { ref mut max, .. } = self;

        max
    }

    /// Decompose the accumulated values into a 2-element tuple.
    #[inline]
    pub const fn into_pair(self) -> (T, T) {
        let Self { min, max } = self;

        (min, max)
    }

    /// Record a single value, without delegating to a specific field.
    ///
    /// This serves as an easy way to record a single value into both the
    /// minimum and maximum fields.
    #[inline]
    pub fn record(&mut self, target_value: T) {
        let &mut Self {
            ref mut min, ref mut max, ..
        } = self;

        *min = (*min).min(target_value);

        *max = (*max).max(target_value);
    }
}

/// A range of lines in a source.
///
/// This is an inclusive range specialized for [`NonZero`] line indices.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Copy)]
pub struct LineRange {
    start: NonZero<usize>,
    end: NonZero<usize>,
}

impl LineRange {
    /// Create a new [`LineRange`] from its raw parts: the start and end
    /// indices.
    #[inline]
    pub const fn from_raw_parts(start: NonZero<usize>, end: NonZero<usize>) -> Self {
        Self { start, end }
    }

    /// Determine the start of this [`LineRange`].
    #[inline]
    pub const fn start(&self) -> NonZero<usize> {
        let &Self { start, .. } = self;

        start
    }

    /// Determine the end of this [`LineRange`].
    #[inline]
    pub const fn end(&self) -> NonZero<usize> {
        let &Self { end, .. } = self;

        end
    }

    /// Reinterpret this [`LineRange`] as a [`RangeInclusive`] of [`usize`]
    /// values.
    #[inline]
    pub const fn range(&self) -> RangeInclusive<usize> {
        let &Self { start, end } = self;

        let start = start.get();

        let end = end.get();

        start..=end
    }
}

impl IntoIterator for LineRange {
    type Item = NonZero<usize>;
    type IntoIter = LineRangeIter;

    fn into_iter(self) -> Self::IntoIter {
        let LineRange { start, end } = self;

        LineRangeIter { current: start, end }
    }
}

/// An iterator over a range of lines.
///
/// See [`LineRange`] for more information.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Copy)]
pub struct LineRangeIter {
    current: NonZero<usize>,
    end: NonZero<usize>,
}

impl Iterator for LineRangeIter {
    type Item = NonZero<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        let &mut Self { ref mut current, end } = self;

        if *current > end {
            return None;
        }

        *current = current.saturating_add(1);

        Some(*current)
    }
}

/// Associated source metadata.
///
/// This piece of information is used to query the general information over a
/// source.
///
/// Precisely, this includes:
///
/// - The [`Span`] that englobes all the relevant lines in the source plane.
/// - An inclusive range of the relevant line indices.
/// - Historical minimum and maximum line lengths for the source.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Metadata {
    /// The [`Span`] over the bytes of the relevant lines.
    line_byte_bounds: Span,

    /// A [`Range`] of the relevant line indices.
    line_range: LineRange,

    /// A [`MinMax`] pair of the historical line lengths for the source.
    line_length: MinMax<usize>,
}

impl Metadata {
    /// Construct a [`Metadata`] piecewise from its individual components.
    #[inline]
    pub const fn from_raw_parts(line_byte_bounds: Span, line_range: LineRange, line_length: MinMax<usize>) -> Self {
        Self {
            line_byte_bounds,
            line_range,
            line_length,
        }
    }
}

impl Metadata {
    /// Retrieve the [`Span`] over the bytes of the relevant lines.
    #[inline]
    pub const fn bytes(&self) -> Span {
        let &Self { line_byte_bounds, .. } = self;

        line_byte_bounds
    }

    /// Retrieve the inclusive range of relevant line indices.
    #[inline]
    pub const fn lines(&self) -> LineRange {
        let &Self { line_range, .. } = self;

        line_range
    }

    /// Retrieve the historical line lengths for the source.
    ///
    /// In other words, this is a method that provides access to
    #[inline]
    pub const fn length(&self) -> MinMax<usize> {
        let &Self { line_length, .. } = self;

        line_length
    }
}
