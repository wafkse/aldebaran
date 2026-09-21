//! Source snippets grouped by annotation contiguity.
//!
//! This module associates annotations with their source lines and groups regions
//! whose line ranges overlap. The iterator types perform this grouping without
//! allocating a separate interval index, allowing renderers to process one
//! related source region at a time.

use core::{fmt, num::NonZero, ops::RangeInclusive};

use aldebaran_print::Print;

use crate::span::Span;
use crate::{error::Lines, error::Source};

use super::Annotated;

/// A printable view of one source and its related annotations.
///
/// Printing discovers contiguous annotation groups, selects the source lines
/// covered by each group, and forwards those lines to the source printing
/// machinery. The view borrows both source and annotations for its full lifetime.
#[derive(Debug)]
pub struct Snippet<'a, S, A>
where
    S: Source<'a>,
    A: Annotated,
{
    target_source: &'a S,
    annotations: (&'a A, &'a [A]),
}

impl<'a, S, A> Snippet<'a, S, A>
where
    S: Source<'a>,
    A: Annotated,
{
    /// Create a new [`Snippet`] with the specified write, target source, total span, and annotations.
    #[inline]
    pub const fn new(target_source: &'a S, annotations: (&'a A, &'a [A])) -> Self {
        Self {
            target_source,
            annotations,
        }
    }
}

impl<'a, S, A> Print for Snippet<'a, S, A>
where
    S: Source<'a>,
    A: Annotated,
{
    type Context = ();
    fn print_with_ctx<W>(&self, writer: &mut W, _: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        let &Self {
            target_source,
            annotations,
        } = self;

        let mut target_iter = Contiguous::new(OneOrMore::pair(annotations), target_source);

        while let Some::<ContiguousFor<'a, A, S>>(target_related) = target_iter.next() {
            // TODO: either figure this shit out or JUST USE ALLOCATION

            let target_lines = {
                let span = target_related.iter().fold(Span::MIN, |acc, annot| acc.superset(annot.span()));

                target_source.lines_between(span)
            };

            target_lines.for_each(|(a, b)| dbg!(a).then(b).print(writer).unwrap());
        }

        Ok(())
    }
}

/// A structure that gurantees that there is at least one [`Annotated`].
#[derive(Debug, Hash)]
pub struct OneOrMore<'a, A>
where
    A: Annotated,
{
    a0: &'a A,
    a_n: &'a [A],
}

impl<'a, A> OneOrMore<'a, A>
where
    A: Annotated,
{
    /// Create a new [`OneOrMore`] with the specified first annotation and the rest of the annotations.
    pub const fn new(a0: &'a A, a_n: &'a [A]) -> Self {
        Self { a0, a_n }
    }

    /// Create a new [`OneOrMore`] with a single annotation.
    #[inline]
    pub const fn single(a0: &'a A) -> Self {
        Self { a0, a_n: &[] }
    }

    /// Create a new [`OneOrMore`] from a single + slice pair.
    #[inline]
    pub const fn pair((a0, a_n): (&'a A, &'a [A])) -> Self {
        Self { a0, a_n }
    }

    /// Retrieve the first annotation.
    #[inline]
    pub const fn first(&self) -> &'a A {
        let &Self { a0, .. } = self;

        a0
    }

    /// Retrieve the rest of the annotations, that is, all annotations except the first one.
    #[inline]
    pub const fn rest(&self) -> &'a [A] {
        let &Self { a_n, .. } = self;

        a_n
    }

    /// Retrieve the `nth` annotation.
    #[inline]
    pub fn nth(&self, n: NonZero<usize>) -> Option<&'a A> {
        let &Self { a_n, .. } = self;

        let target_index = n.get() - 1;

        a_n.get(target_index)
    }

    /// Retrieve an iterator over all annotations.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &'a A> {
        let &Self { a0, a_n } = self;

        core::iter::once(a0).chain(a_n.iter())
    }
}

impl<'a, A> Clone for OneOrMore<'a, A>
where
    A: Annotated,
{
    #[inline]
    fn clone(&self) -> Self {
        let &Self { a0, a_n } = self;

        Self { a0, a_n }
    }
}

impl<'a, A> Copy for OneOrMore<'a, A> where A: Annotated {}

/// Iterator state for traversing a guaranteed nonempty annotation collection.
///
/// The first annotation is yielded without an index lookup, then the iterator
/// advances through the remaining slice using nonzero positions.
#[derive(Debug, Hash)]
pub struct OnceOrMore<'a, A>
where
    A: Annotated,
{
    i0: Option<NonZero<usize>>,
    a0: OneOrMore<'a, A>,
}

impl<'a, A> Iterator for OnceOrMore<'a, A>
where
    A: Annotated,
{
    type Item = &'a A;

    fn next(&mut self) -> Option<Self::Item> {
        let &mut Self { ref mut i0, a0 } = self;

        match i0 {
            Some(target_index) => {
                let target_value = a0.nth(*target_index)?;

                let _ = target_index.checked_add(1)?;

                Some(target_value)
            }
            None => {
                let _ = i0.replace(NonZero::<usize>::MIN);

                Some(a0.first())
            }
        }
    }
}

/// Iterator that discovers one annotation region at a time.
///
/// Annotations belong to the same region when their source line ranges overlap.
/// Each yielded [`ContiguousFor`] becomes the query view for one region, while
/// previously grouped annotations are skipped on later iterations.
#[derive(Debug, Hash)]
pub struct Contiguous<'a, A, S>
where
    A: Annotated,
    S: Source<'a>,
{
    /// The collection of annotations to query for contiguity.
    a0: OneOrMore<'a, A>,
    /// The index of the current annotation.
    i0: Option<NonZero<usize>>,
    /// The source of the annotations.
    source: &'a S,
}

impl<'a, A, S> Contiguous<'a, A, S>
where
    A: Annotated,
    S: Source<'a>,
{
    /// Create a new [`Contiguous`] iterator with the specified annotations and source.
    pub const fn new(a0: OneOrMore<'a, A>, source: &'a S) -> Self {
        Self { a0, i0: None, source }
    }
}

/// [`Iterator`] implementation for [`Contiguous`].
///
/// This iterator yields [`ContiguousFor`] instances, which are used to query for contiguous annotations.
///
/// # Time Complexity
///
/// The time complexity is `O(n + (n - 1)^2 + 1)`, due to repeated iteration over the annotations to determine contiguity between and the
/// ausence of allocation.
impl<'a, A, S> Iterator for Contiguous<'a, A, S>
where
    A: Annotated,
    S: Source<'a>,
{
    // FIXME: use impl Iterator instead of ContiguousFor when stable
    type Item = ContiguousFor<'a, A, S>;

    fn next(&mut self) -> Option<Self::Item> {
        let &mut Self {
            ref a0,
            ref mut i0,
            source,
        } = self;

        match i0 {
            Some(target_index) => {
                let current_index = *target_index;

                let target_value = a0.nth(current_index)?;

                let target_range = source.lines_between(target_value.span()).range();

                // Either contiguous or overlapping.
                let is_contiguous = a0.iter().take(target_index.get()).any(|a1| {
                    let r0 = source.lines_between(a1.span()).range();

                    r0.start() <= target_range.end() && r0.end() >= target_range.start()
                });

                if is_contiguous {
                    // Has been previously deemed contiguous, must skip.
                    let _ = target_index.checked_add(1)?;

                    self.next()
                } else {
                    let target_iter = ContiguousFor {
                        inflection_point: Some(*target_index),
                        target_annotation: target_value,
                        target_range,
                        annotations: *a0,
                        source,
                    };

                    let _ = target_index.checked_add(1)?;

                    Some(target_iter)
                }
            }
            None => {
                let _ = i0.replace(NonZero::<usize>::MIN);

                // *NOTE: No contiguity check is done here, as the first annotation is always guaranteed to not have been previously deemed
                // contiguous.
                let target_value = a0.first();

                let target_range = source.lines_between(target_value.span()).range();

                let target_iter = ContiguousFor {
                    inflection_point: None,
                    target_annotation: target_value,
                    annotations: *a0,
                    target_range,
                    source,
                };

                Some(target_iter)
            }
        }
    }
}

/// Query view centered on one annotation region.
///
/// The view records the target annotation and its line range, then exposes an
/// iterator over every annotation that overlaps that range. The inflection point
/// prevents rescanning annotations already known to precede the region.
#[derive(Debug, Clone, Hash)]
pub struct ContiguousFor<'a, A, S>
where
    A: Annotated,
    S: Source<'a>,
{
    /// The index the target annotation is at.
    ///
    /// This is used to optimize iteration by skipping elements known to not be contiguous (the preceding ones).
    inflection_point: Option<NonZero<usize>>,
    /// The target annotation, that is, the annotation that is being queried for.
    target_annotation: &'a A,
    /// The range of lines the target annotation spans.
    target_range: RangeInclusive<NonZero<usize>>,
    /// A collection of all annotations that can be contiguous with the target annotation.
    annotations: OneOrMore<'a, A>,
    /// The source of the annotations.
    source: &'a S,
}

impl<'a, A, S> ContiguousFor<'a, A, S>
where
    A: Annotated,
    S: Source<'a>,
{
    /// Retrieve the range of lines the target annotation spans.
    #[inline]
    pub const fn range(&self) -> &RangeInclusive<NonZero<usize>> {
        let &Self { ref target_range, .. } = self;

        target_range
    }

    /// Retrieve the target annotation, that is, the annotation centric to the iteration.
    #[inline]
    pub const fn target(&self) -> &'a A {
        let &Self { target_annotation, .. } = self;

        target_annotation
    }

    /// Iterate over all contiguous annotations for the target annotation.
    #[inline]
    pub fn iter(&self) -> ContiguousIterator<'a, A, S> {
        let &Self {
            inflection_point,
            target_annotation,
            ref target_range,
            annotations,
            source,
        } = self;

        let target_range = target_range.clone();

        ContiguousIterator {
            inflection_point,
            cutoff_point: None,
            target_annotation,
            target_range,
            annotations,
            source,
        }
    }
}

/// Iterator over annotations that overlap one target line range.
///
/// Iteration resumes from the last discovered position and skips earlier entries
/// using the region inflection point. This preserves the allocation-free grouping
/// strategy used by [`Contiguous`].
#[derive(Debug, Clone, Hash)]
pub struct ContiguousIterator<'a, A, S>
where
    A: Annotated,
    S: Source<'a>,
{
    /// The index the target annotation is at.
    ///
    /// This is used to optimize iteration by skipping elements known to not be contiguous (the preceding ones).
    inflection_point: Option<NonZero<usize>>,
    /// The point where the iteration is currently at.
    cutoff_point: Option<NonZero<usize>>,
    /// The target annotation, that is, the annotation that is being queried for.
    target_annotation: &'a A,
    /// The range of lines the target annotation spans.
    target_range: RangeInclusive<NonZero<usize>>,
    /// A collection of all annotations that can be contiguous with the target annotation.
    annotations: OneOrMore<'a, A>,
    /// The source of the annotations.
    source: &'a S,
}

impl<'a, A, S> Iterator for ContiguousIterator<'a, A, S>
where
    A: Annotated,
    S: Source<'a>,
{
    type Item = &'a A;

    fn next(&mut self) -> Option<Self::Item> {
        let &mut Self {
            ref mut inflection_point,
            ref mut cutoff_point,
            target_annotation: _,
            ref target_range,
            ref annotations,
            source,
        } = self;

        let mut target_iter = annotations
            .iter()
            .skip(
                cutoff_point
                    .map(NonZero::get)
                    .unwrap_or_else(|| inflection_point.map(NonZero::get).unwrap_or(usize::MIN)),
            )
            .enumerate();

        loop {
            let (target_index, target_value) = target_iter.next()?;

            let found_range = source.lines_between(target_value.span()).range();

            // FIXME: unify line ranges into a separate type and consequently implement a custom contiguity check
            let is_contiguous = found_range.start() <= target_range.end() && found_range.end() >= target_range.start();

            if is_contiguous {
                let _ = inflection_point.replace(NonZero::new(target_index)?);

                break Some(target_value);
            }
        }
    }
}
