//! Half-open ranges passed between source-processing components.
//!
//! [`Span`] records coordinates without identifying their source or unit. The
//! producer and consumer must agree on those coordinates, such as UTF-8 bytes
//! within the same source. Endpoints saturate on overflow, so a nonzero stored
//! length can describe fewer addressable coordinates near `usize::MAX`.

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

            let length = match NonZero::new(end - start) {
                Some(length) => length,
                None => return None,
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
