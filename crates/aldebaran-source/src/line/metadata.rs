//! Aggregate metadata for source line extents.
//!
//! See [`Metadata`] for further information.

use core::{num::NonZero, ops::RangeInclusive};

use aldebaran_baseline::prelude::Span;
use num_traits::PrimInt;

/// Seals supported integer record types to the built-in primitive set.
mod private {
    /// Restricts source records to built-in integer representations.
    pub trait Sealed {}
}

/// Implements the source line record marker for selected built-in integers.
macro_rules! recordable {
    ($($target_type:ty),+ $(,)?) => {
        $(impl private::Sealed for $target_type {})+
    };
}

recordable!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);

/// Trait for recordable primitives.
///
/// A primitive is considered recordable if it can be used to record historical
/// minimum and maximum values.
///
/// In other words, a recordable primitive must be copyable and comparable with
/// a total order.
pub trait Record: PrimInt + private::Sealed + Copy + PartialOrd + Ord {}

impl<R> Record for R where R: PrimInt + private::Sealed + Copy + PartialOrd + Ord {}

/// A pair of historical minimum and maximum values for a target,
/// [`recordable`](Record) primitive.
///
/// [`record`](Self::record) expands the bounds to include each observation.
/// Raw construction and mutable access do not enforce ordering. Extrema
/// accumulation requires coherent bounds.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Default)]
// NOTE(invariant): None.
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
    /// The inclusive interval from the stored minimum to the stored maximum.
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
    ///
    /// The bounds are stored without reordering them. [`MinMax::range`] is empty when `min > max`.
    #[inline]
    pub const fn pair((min, max): (T, T)) -> Self {
        Self { min, max }
    }

    /// Retrieve the minimum value recorded.
    ///
    /// In line metadata, this is the shortest span length, including its terminator.
    #[inline]
    pub const fn min(&self) -> T {
        let &Self { min, .. } = self;

        min
    }

    /// Retrieve the maximum value recorded.
    ///
    /// In line metadata, this is the longest span length, including its terminator.
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

    /// Adjust the lower bound directly without reordering it against the upper bound.
    #[inline]
    pub const fn min_mut(&mut self) -> &mut T {
        let &mut Self { ref mut min, .. } = self;

        min
    }

    /// Adjust the upper bound directly without reordering it against the lower bound.
    #[inline]
    pub const fn max_mut(&mut self) -> &mut T {
        let &mut Self { ref mut max, .. } = self;

        max
    }

    /// Decompose the accumulated values into a 2-element tuple.
    #[inline]
    pub const fn partitioned(self) -> (T, T) {
        let Self { min, max } = self;

        (min, max)
    }

    /// Record a single value, without delegating to a specific field.
    ///
    /// This expands the bounds to include the value.
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
///
/// A reversed range is empty when viewed through [`LineRange::range`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Copy)]
// NOTE(invariant): None.
pub struct LineRange {
    /// The inclusive first line number.
    start: NonZero<usize>,

    /// The inclusive final line number.
    end: NonZero<usize>,
}

impl LineRange {
    /// Create a new [`LineRange`] from its raw parts: the start and end
    /// indices.
    ///
    /// The inclusive bounds are stored without reordering them. A reversed
    /// range is empty when viewed through [`LineRange::range`].
    #[inline]
    pub const fn raw(start: NonZero<usize>, end: NonZero<usize>) -> Self {
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
    ///
    /// Reversed bounds describe an empty range.
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

        LineRangeIter { current: Some(start), end }
    }
}

/// An iterator over a range of lines.
///
/// See [`LineRange`] for more information. The range can include `usize::MAX`,
/// but yields no values if its bounds are reversed. Once exhausted, the iterator
/// continues to return `None`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Copy)]
// NOTE(invariant): None.
pub struct LineRangeIter {
    /// The line-number cursor advanced by iteration, absent after exhaustion.
    current: Option<NonZero<usize>>,

    /// The inclusive final line number.
    end: NonZero<usize>,
}

impl Iterator for LineRangeIter {
    type Item = NonZero<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        let &mut Self { ref mut current, end } = self;

        let value = (*current)?;

        if value > end {
            *current = None;
            return None;
        }

        *current = if value == end { None } else { value.checked_add(1) };

        Some(value)
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
///
/// Line spans and lengths include their terminators and use source-coordinate
/// units.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
// NOTE(invariant): (ByteBounds, LineRange, LengthExtrema) = Describe(SameLineSelection)
pub struct Metadata {
    /// The [`Span`] over the bytes of the relevant lines.
    line_byte_bounds: Span,

    /// A [`LineRange`] of the relevant line indices.
    line_range: LineRange,

    /// A [`MinMax`] pair of the historical line lengths for the source.
    line_length: MinMax<usize>,
}

impl Metadata {
    /// Construct a [`Metadata`] piecewise from its individual components.
    ///
    /// The bounds and statistics must describe the same line selection.
    #[inline]
    pub const fn raw(line_byte_bounds: Span, line_range: LineRange, line_length: MinMax<usize>) -> Self {
        Self {
            line_byte_bounds,
            line_range,
            line_length,
        }
    }
}

impl Metadata {
    /// Retrieve the [`Span`] over the bytes of the relevant lines.
    ///
    /// Coordinates count UTF-8 bytes for strings and bytes for byte slices, and
    /// the selected line spans include their terminators.
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
    /// These lengths include line terminators and are measured in source
    /// coordinates rather than terminal display width.
    #[inline]
    pub const fn length(&self) -> MinMax<usize> {
        let &Self { line_length, .. } = self;

        line_length
    }
}

#[cfg(test)]
mod tests {
    //! Tests module for the [`metadata`] module.
    //!
    //! [`metadata`]: super

    use alloc::vec::Vec;
    use core::num::NonZero;

    use super::LineRange;

    #[test]
    fn inclusive_ranges_terminate_at_ordinary_and_maximum_bounds() {
        for (start, end, expected) in [
            (1, 1, &[1][..]),
            (2, 4, &[2, 3, 4][..]),
            (4, 2, &[][..]),
            (usize::MAX, usize::MAX, &[usize::MAX][..]),
            (usize::MAX - 1, usize::MAX, &[usize::MAX - 1, usize::MAX][..]),
            (usize::MAX, usize::MAX - 1, &[][..]),
        ] {
            let range = LineRange::raw(
                NonZero::new(start).expect("positive start"),
                NonZero::new(end).expect("positive end"),
            );
            let mut iter = range.into_iter();
            assert_eq!(
                iter.by_ref().take(expected.len() + 1).map(NonZero::get).collect::<Vec<_>>(),
                expected
            );
            assert_eq!(iter.next(), None);
            assert_eq!(iter.next(), None);
        }
    }
}
