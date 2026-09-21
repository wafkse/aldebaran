//! The span module provides a simple abstraction over the concept of a span.
//!
//! See the [`Span`] struct for more information.

use core::{fmt, num::NonZero, ops::Range};

use aldebaran_print::prelude::Print;

/// A nonempty half-open coordinate span.
///
/// A span represents `[start, end)` and is always at least one element long.
/// The type does not prove that its coordinates are contained by any particular
/// source. Source-aware APIs define and validate their own containment rules.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub struct Span {
    /// The inclusive start coordinate of the span.
    start: usize,

    /// The nonzero length of the half-open span.
    length: NonZero<usize>,
}

// NOTE(invariant): Every `Span` has nonzero length and uses half-open
// coordinates. Source containment is deliberately not encoded by this type.

impl Span {
    /// A span that is, technically, at the end of the world.
    ///
    /// This is an unit span that is both maximum in length and maximum in start
    /// index.
    pub const END_OF_WORLD: Self = Self {
        start: usize::MAX,
        length: NonZero::<usize>::MAX,
    };

    /// A span that always contains all the elements.
    pub const ALL: Self = Self {
        start: usize::MIN,
        length: NonZero::<usize>::MAX,
    };

    /// The minimum possible span from a numerical standpoint, i.e, a span of
    /// length `1`, or an unit span.
    ///
    /// Starts at `0`.
    pub const MIN: Self = Self {
        start: usize::MIN,
        length: NonZero::<usize>::MIN,
    };

    /// Create a new [`Span`] of the target length.
    #[inline]
    #[must_use]
    pub const fn new(start: usize, length: NonZero<usize>) -> Self {
        Self { start, length }
    }

    /// Create a new [`Span`] of length `1`, i.e, an unit span.
    #[inline]
    #[must_use]
    pub const fn unit(start: usize) -> Self {
        let length = NonZero::<usize>::MIN;

        Self { start, length }
    }

    /// Retrieve the position of the start of the span.
    ///
    /// # Time Complexity
    ///
    /// This operation is `O(1)`.
    #[inline]
    #[must_use]
    pub const fn start(&self) -> usize {
        let &Self { start, .. } = self;

        start
    }

    /// Retrieve the position of the end of the span.
    ///
    /// Note that this is the position where the span ends, that is, it is
    /// exclusive.
    ///
    /// # Time Complexity
    ///
    /// This operation is `O(1)`.
    ///
    /// # Considerations
    ///
    /// If the combined value of `start` and `length` would overflow, this
    /// function will saturate at the numeric limit.
    #[inline]
    #[must_use]
    pub const fn end(&self) -> usize {
        let &Self { start, length } = self;

        start.saturating_add(length.get())
    }

    /// Retrieve the length of the span.
    ///
    /// # Time Complexity
    ///
    /// This operation is `O(1)`.
    #[inline]
    #[must_use]
    #[doc(alias = "size")]
    pub const fn length(&self) -> NonZero<usize> {
        let &Self { length, .. } = self;

        length
    }

    /// Determine if the span contains the target index.
    ///
    /// # Time Complexity
    ///
    /// This operation is `O(1)`.
    #[inline]
    #[must_use]
    #[doc(alias = "has")]
    pub const fn contains(&self, target_index: usize) -> bool {
        let &Self { start, .. } = self;

        target_index >= start && target_index < self.end()
    }

    /// Determine if the current [`Span`] is fully contained within the target
    /// [`Span`].
    ///
    /// # Time Complexity
    ///
    /// This operation is `O(1)`.
    #[inline]
    #[must_use]
    #[doc(alias = "inside_of")]
    pub const fn within(&self, target_span: Self) -> bool {
        let &Self { start, .. } = self;

        let end = self.end();

        start >= target_span.start && end <= target_span.end()
    }

    /// Determine if the current [`Span`] intersects with the target [`Span`].
    /// That is, if the two spans have at least one element in common.
    ///
    /// # Time Complexity
    ///
    /// This operation is `O(1)`.
    ///
    /// # Examples
    #[inline]
    #[must_use]
    pub const fn overlaps(&self, target_span: Self) -> bool {
        let &Self { start, .. } = self;

        let end = self.end();

        let target_end = target_span.end();

        start < target_end && end > target_span.start
    }

    /// Retrieve the superset, i.e, the smallest span that contains both the
    /// current and target [`Span`]s.
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

    /// Determine the *intersection* between the two spans.
    ///
    /// If the two spans do not intersect, the result is [`None`].
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

            let length = match NonZero::new(end - start) {
                Some(length) => length,
                None => return None,
            };

            Some(Self { start, length })
        } else {
            None
        }
    }

    /// Determine if the current [`span`](Span) is a subset of the target
    /// [`span`](Span).
    #[inline]
    pub const fn englobes(&self, target_span: Self) -> bool {
        let &Self { start, .. } = self;

        let end = self.end();

        let target_end = target_span.end();

        start <= target_span.start && end >= target_end
    }

    /// Reinterpret the current [`Span`] as a [`half-open range`](Range).
    #[inline]
    pub const fn range(&self) -> Range<usize> {
        let &Self { start, .. } = self;

        start..self.end()
    }

    /// Normalize the current to start from `0`.
    ///
    /// This is indeed useful in cases where relative spans are required.
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
    fn print_with_ctx<W>(&self, writer: &mut W, _: &Self::Context) -> fmt::Result
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
