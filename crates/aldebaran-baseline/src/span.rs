//! Half-open ranges passed between source-processing components.
//!
//! [`Span`] records coordinates without identifying their source or unit. The
//! producer and consumer must agree on those coordinates, such as UTF-8 bytes
//! within the same source. Endpoints saturate on overflow, so a nonzero stored
//! length can describe fewer addressable coordinates near `usize::MAX`.

mod maybe_spanned;
mod spanned;

pub use self::{maybe_spanned::MaybeSpanned, spanned::Spanned};

use core::{fmt, num::NonZero, ops::Range};

use aldebaran_print::prelude::Print;

/// A nonempty half-open range in caller-defined coordinates.
///
/// The range covers `[start, end)`, with its nonzero length measured in the
/// same unit as `start`. Because a span carries no source identity, however,
/// its coordinates alone cannot establish which source they belong to.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, Ord, PartialOrd)]
// NOTE(invariant): None.
pub struct Span {
    /// The inclusive start coordinate of the span.
    start: usize,

    /// The nonzero length of the half-open span.
    length: NonZero<usize>,
}

impl Span {
    /// A saturation sentinel at the maximum coordinate.
    ///
    /// Its start and saturated exclusive endpoint are both `usize::MAX`, so
    /// its half-open range contains no index, including `usize::MAX`.
    pub const END_OF_WORLD: Self = Self {
        start: usize::MAX,
        length: NonZero::<usize>::MAX,
    };

    /// The widest representable span starting at zero.
    ///
    /// Its exclusive endpoint is `usize::MAX`, so it does not contain that
    /// coordinate.
    pub const ALL: Self = Self {
        start: usize::MIN,
        length: NonZero::<usize>::MAX,
    };

    /// The one-coordinate span starting at zero.
    pub const MIN: Self = Self {
        start: usize::MIN,
        length: NonZero::<usize>::MIN,
    };

    /// Create a span in the caller's coordinate unit.
    ///
    /// The constructor permits `start + length` to overflow. [`Self::end`]
    /// then saturates at `usize::MAX`.
    #[inline]
    #[must_use]
    pub const fn new(start: usize, length: NonZero<usize>) -> Self {
        Self { start, length }
    }

    /// Create a span covering one coordinate at `start`.
    #[inline]
    #[must_use]
    pub const fn unit(start: usize) -> Self {
        let length = NonZero::<usize>::MIN;

        Self { start, length }
    }

    /// Inclusive lower bound in the caller's coordinate system.
    #[inline]
    #[must_use]
    pub const fn start(&self) -> usize {
        let &Self { start, .. } = self;

        start
    }

    /// Return the exclusive end coordinate, saturating at `usize::MAX` on overflow.
    #[inline]
    #[must_use]
    pub const fn end(&self) -> usize {
        let &Self { start, length } = self;

        start.saturating_add(length.get())
    }

    /// Stored extent in the caller's coordinate unit.
    ///
    /// When the endpoint saturates, this can exceed `end() - start()`.
    #[inline]
    #[must_use]
    #[doc(alias = "size")]
    pub const fn length(&self) -> NonZero<usize> {
        let &Self { length, .. } = self;

        length
    }

    /// Test whether a coordinate lies in the half-open range.
    #[inline]
    #[must_use]
    #[doc(alias = "has")]
    pub const fn contains(&self, target_index: usize) -> bool {
        let &Self { start, .. } = self;

        target_index >= start && target_index < self.end()
    }

    /// Test whether this range is fully contained within `target_span`.
    #[inline]
    #[must_use]
    #[doc(alias = "inside_of")]
    pub const fn within(&self, target_span: Self) -> bool {
        let &Self { start, .. } = self;

        let end = self.end();

        start >= target_span.start && end <= target_span.end()
    }

    /// Test whether the ranges share at least one coordinate.
    #[inline]
    #[must_use]
    pub const fn overlaps(&self, target_span: Self) -> bool {
        let &Self { start, .. } = self;

        let end = self.end();

        let target_end = target_span.end();

        start < target_end && end > target_span.start
    }

    /// Return the smallest span containing both ranges.
    ///
    /// # Panics
    ///
    /// Panics when both spans start at `usize::MAX`.
    #[inline]
    #[must_use]
    #[doc(alias = "join")]
    pub const fn superset(&self, target_other: Self) -> Self {
        // FIXME: get rid of these helper functions once Ord has const fn min
        // and max
        #[inline]
        const fn min(a: usize, b: usize) -> usize {
            if a < b { a } else { b }
        }

        #[inline]
        const fn max(a: usize, b: usize) -> usize {
            if a > b { a } else { b }
        }

        let start = min(self.start(), target_other.start());
        let end = max(self.end(), target_other.end());

        let length = NonZero::new(end - start).expect("length is never zero");

        Self { start, length }
    }

    /// Shared coordinates of both ranges, or `None` when they do not overlap.
    #[inline]
    #[must_use]
    pub const fn intersect(&self, other: Self) -> Option<Self> {
        // FIXME: get rid of these helper functions once Ord has const fn min
        // and max
        #[inline]
        const fn min(a: usize, b: usize) -> usize {
            if a < b { a } else { b }
        }

        #[inline]
        const fn max(a: usize, b: usize) -> usize {
            if a > b { a } else { b }
        }

        if self.start() < other.end() && self.end() > other.start() {
            let start = max(self.start(), other.start());
            let end = min(self.end(), other.end());

            let Some(length) = NonZero::new(end - start) else {
                return None;
            };

            Some(Self { start, length })
        } else {
            None
        }
    }

    /// Containment of `target_span`, permitting equal lower and upper boundaries.
    #[inline]
    pub const fn englobes(&self, target_span: Self) -> bool {
        let &Self { start, .. } = self;

        let end = self.end();

        let target_end = target_span.end();

        start <= target_span.start && end >= target_end
    }

    /// Return the half-open range using the saturated endpoint from [`end`](Self::end).
    ///
    /// A span starting at `usize::MAX` therefore produces an empty range.
    #[inline]
    pub const fn range(&self) -> Range<usize> {
        let &Self { start, .. } = self;

        start..self.end()
    }

    /// Move the start coordinate to zero while preserving the length.
    #[inline]
    pub const fn normal(&self) -> Self {
        let &Self { length, .. } = self;

        let start = 0;

        Self { start, length }
    }
}

impl Print for Span {
    type Context = ();
    #[inline]
    fn print_with_context<W>(&self, writer: &mut W, _: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        let &Self { start: s0, .. } = self;

        let e0 = self.end();

        write!(writer, "{}:{}", s0, e0)
    }
}

impl fmt::Display for Span {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.start(), self.end())
    }
}

#[cfg(test)]
mod tests {
    use core::num::NonZeroUsize;

    use super::Span;

    #[test]
    fn span_new() {
        let length = NonZeroUsize::new(5).expect("test length is nonzero");
        let span = Span::new(10, length);
        assert_eq!(span.start(), 10);
        assert_eq!(span.length(), length);
    }

    #[test]
    fn span_unit() {
        let span = Span::unit(10);
        assert_eq!(span.start(), 10);
        assert_eq!(span.length().get(), 1);
    }

    #[test]
    fn span_start() {
        let length = NonZeroUsize::new(5).expect("test length is nonzero");
        let span = Span::new(10, length);
        assert_eq!(span.start(), 10);
    }

    #[test]
    fn span_end() {
        let length = NonZeroUsize::new(5).expect("test length is nonzero");
        let span = Span::new(10, length);
        assert_eq!(span.end(), 15);
    }

    #[test]
    fn span_contains() {
        let length = NonZeroUsize::new(5).expect("test length is nonzero");
        let span = Span::new(10, length);
        assert!(span.contains(12));
        assert!(!span.contains(15));
    }

    #[test]
    fn span_within() {
        let length1 = NonZeroUsize::new(5).expect("test length is nonzero");
        let span1 = Span::new(10, length1);
        let length2 = NonZeroUsize::new(10).expect("test length is nonzero");
        let span2 = Span::new(5, length2);
        assert!(span1.within(span2));
        assert!(!span2.within(span1));
    }

    #[test]
    fn span_overlaps() {
        let length1 = NonZeroUsize::new(5).expect("test length is nonzero");
        let span1 = Span::new(10, length1);
        let length2 = NonZeroUsize::new(10).expect("test length is nonzero");
        let span2 = Span::new(12, length2);
        assert!(span1.overlaps(span2));
        assert!(span2.overlaps(span1));
    }

    #[test]
    fn span_superset() {
        let length1 = NonZeroUsize::new(5).expect("test length is nonzero");
        let span1 = Span::new(10, length1);
        let length2 = NonZeroUsize::new(10).expect("test length is nonzero");
        let span2 = Span::new(5, length2);
        let superset = span1.superset(span2);
        assert_eq!(superset.start(), 5);
        assert_eq!(superset.end(), 15);
    }

    #[test]
    fn span_intersect() {
        let length1 = NonZeroUsize::new(5).expect("test length is nonzero");
        let span1 = Span::new(10, length1);
        let length2 = NonZeroUsize::new(10).expect("test length is nonzero");
        let span2 = Span::new(12, length2);
        let intersection = span1.intersect(span2).expect("test spans overlap");

        assert_eq!(intersection.start(), 12);
        assert_eq!(intersection.end(), 15);
    }

    #[test]
    fn span_englobes() {
        let length1 = NonZeroUsize::new(10).expect("test length is nonzero");
        let span1 = Span::new(5, length1);
        let length2 = NonZeroUsize::new(5).expect("test length is nonzero");
        let span2 = Span::new(10, length2);
        assert!(span1.englobes(span2));
        assert!(!span2.englobes(span1));
    }

    #[test]
    fn span_range() {
        let length = NonZeroUsize::new(5).expect("test length is nonzero");
        let span = Span::new(10, length);
        let range = span.range();
        assert_eq!(range.start, 10);
        assert_eq!(range.end, 15);
    }

    #[test]
    fn touching_and_disjoint_spans_have_no_intersection() {
        let left = Span::new(2, NonZeroUsize::new(3).expect("nonzero length"));

        assert!(!left.contains(1));
        assert!(left.contains(2));
        assert!(left.contains(4));
        assert!(!left.contains(5));

        for right in [Span::unit(5), Span::unit(7)] {
            assert!(!left.overlaps(right));
            assert!(!right.overlaps(left));
            assert_eq!(left.intersect(right), None);
            assert_eq!(right.intersect(left), None);
            assert!(!left.within(right));
            assert!(!left.englobes(right));
        }
    }

    #[test]
    fn equal_and_nested_spans_preserve_set_relationships() {
        let outer = Span::new(2, NonZeroUsize::new(6).expect("nonzero length"));
        let inner = Span::new(3, NonZeroUsize::new(2).expect("nonzero length"));

        for contained in [outer, inner] {
            assert!(contained.within(outer));
            assert!(outer.englobes(contained));
            assert_eq!(outer.intersect(contained), Some(contained));
            assert_eq!(contained.intersect(outer), Some(contained));
            assert_eq!(outer.superset(contained), outer);
            assert_eq!(contained.superset(outer), outer);
        }
    }

    #[test]
    fn saturated_spans_exclude_the_maximum_coordinate() {
        let span = Span::new(usize::MAX - 2, NonZeroUsize::new(5).expect("nonzero length"));

        assert_eq!(span.end(), usize::MAX);
        assert_eq!(span.length().get(), 5);
        assert!(span.contains(usize::MAX - 1));
        assert!(!span.contains(usize::MAX));
        assert_eq!(span.range(), usize::MAX - 2..usize::MAX);
        assert_eq!(span.normal().range(), 0..5);
        assert_eq!(span.intersect(Span::ALL).map(|value| value.range()), Some(span.range()));
        assert_eq!(span.intersect(Span::END_OF_WORLD), None);
        assert!(!Span::END_OF_WORLD.overlaps(span));
        assert!(Span::END_OF_WORLD.range().is_empty());
        assert!(!Span::END_OF_WORLD.contains(usize::MAX));
        assert_eq!(Span::ALL.range(), 0..usize::MAX);
        assert_eq!(Span::MIN, Span::unit(0));
    }

    #[test]
    #[should_panic(expected = "length is never zero")]
    fn superset_of_two_maximum_starts_panics() {
        let _span = Span::END_OF_WORLD.superset(Span::unit(usize::MAX));
    }
}
