//! Bounds used to label and size a source-line selection.
//!
//! [`Metadata`] combines an absolute source extent, inclusive [`LineRange`],
//! and [`MinMax`] line lengths. Lengths include terminators and use source
//! coordinate units, so UTF-8 byte lengths are distinct from display widths.

use core::{num::NonZero, ops::RangeInclusive};

use aldebaran_baseline::prelude::Span;
use num_traits::PrimInt;

/// Restricts records to supported built-in integer types.
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

/// Built-in integer types supported by line statistics.
pub trait Record: PrimInt + private::Sealed + Copy + PartialOrd + Ord {}

impl<R> Record for R where R: PrimInt + private::Sealed + Copy + PartialOrd + Ord {}

/// Bounds that can accumulate extrema across integer observations.
///
/// [`record`](Self::record) expands the bounds to include each observation.
/// Raw construction and mutable access do not enforce ordering, so callers
/// using this as an extrema accumulator must initialize coherent bounds.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Default)]
// NOTE(invariant): None.
pub struct MinMax<T = usize>
where
    T: Record,
{
    /// Lower observation bound, or the lower value supplied by the caller.
    min: T,

    /// Upper observation bound, or the upper value supplied by the caller.
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
    /// Seed an extrema accumulator with its first observation.
    #[inline]
    pub const fn initial(value: T) -> Self {
        let min = value;
        let max = value;

        Self { min, max }
    }

    /// Store bounds without reordering them.
    ///
    /// [`MinMax::range`] is empty when `min > max`.
    #[inline]
    pub const fn pair((min, max): (T, T)) -> Self {
        Self { min, max }
    }

    /// Lower endpoint in the same units as the recorded observations.
    /// In line metadata this is the shortest span length including its terminator.
    #[inline]
    pub const fn min(&self) -> T {
        let &Self { min, .. } = self;

        min
    }

    /// Upper endpoint in the same units as the recorded observations.
    /// In line metadata this is the longest span length including its terminator.
    #[inline]
    pub const fn max(&self) -> T {
        let &Self { max, .. } = self;

        max
    }

    /// Lower endpoint of the inclusive interval exposed by [`Self::range`].
    #[inline]
    pub const fn min_ref(&self) -> &T {
        let &Self { ref min, .. } = self;

        min
    }

    /// Upper endpoint of the inclusive interval exposed by [`Self::range`].
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

    /// Consume the bounds as `(minimum, maximum)`.
    #[inline]
    pub const fn into_pair(self) -> (T, T) {
        let Self { min, max } = self;

        (min, max)
    }

    /// Expand the interval to include another observation.
    #[inline]
    pub fn record(&mut self, target_value: T) {
        let &mut Self {
            ref mut min, ref mut max, ..
        } = self;

        *min = (*min).min(target_value);

        *max = (*max).max(target_value);
    }
}

/// Inclusive line-number bounds.
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
    /// Store the inclusive bounds without reordering them.
    #[inline]
    pub const fn from_raw_parts(start: NonZero<usize>, end: NonZero<usize>) -> Self {
        Self { start, end }
    }

    /// Original line number at the selection's inclusive lower boundary.
    #[inline]
    pub const fn start(&self) -> NonZero<usize> {
        let &Self { start, .. } = self;

        start
    }

    /// Original line number at the selection's inclusive upper boundary.
    #[inline]
    pub const fn end(&self) -> NonZero<usize> {
        let &Self { end, .. } = self;

        end
    }

    /// Line labels covered by the selection, with both endpoints included.
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

/// Traversal of inclusive line numbers, including a possible `usize::MAX` endpoint.
/// Reversed bounds yield no values and exhaustion remains permanent.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Copy)]
// NOTE(invariant): None.
pub struct LineRangeIter {
    /// Next candidate line number, absent after exhaustion.
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

/// Source span, line numbers, and length extrema for a source line selection.
///
/// Line spans and lengths include their terminators and use source-coordinate
/// units.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
// NOTE(invariant): (ByteBounds, LineRange, LengthExtrema) = Describe(SameLineSelection)
pub struct Metadata {
    /// Absolute extent from the first line's start through the final line's break or content end.
    line_byte_bounds: Span,

    /// The inclusive line-number bounds associated with the selection.
    line_range: LineRange,

    /// Minimum and maximum line-span lengths, including terminators.
    line_length: MinMax<usize>,
}

impl Metadata {
    /// Store the supplied bounds and statistics. The caller must ensure they
    /// describe the same line selection.
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
    /// Absolute extent of the selected lines, including their terminators.
    /// Coordinates count UTF-8 bytes for strings and bytes for byte slices.
    #[inline]
    pub const fn bytes(&self) -> Span {
        let &Self { line_byte_bounds, .. } = self;

        line_byte_bounds
    }

    /// Original first and last line numbers, both included in the selection.
    #[inline]
    pub const fn lines(&self) -> LineRange {
        let &Self { line_range, .. } = self;

        line_range
    }

    /// Shortest and longest selected line extents in source units, including breaks.
    /// These lengths do not measure terminal display width.
    #[inline]
    pub const fn length(&self) -> MinMax<usize> {
        let &Self { line_length, .. } = self;

        line_length
    }
}

#[cfg(test)]
mod tests {
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
            let range = LineRange::from_raw_parts(
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
