//! Incremental line discovery over iterable sources.
//!
//! [`LineIter`] feeds source components into one terminator state machine and
//! yields absolute [`LineId`] values as breaks are recognized. The iterator tracks
//! both byte offset and one-based line number across successive source views.

use core::{cmp::Ordering, fmt, num::NonZero};

use aldebaran_span::prelude::{Span, Spanned};

use crate::{
    component::Component,
    iter::SourceIter,
    line::{LineContent, LineId, SourceLines},
    terminate::{Outcome, Terminator, TerminatorContext},
};

/// Stateful iterator that discovers lines with one terminator policy.
///
/// Each step begins at the current absolute source offset and feeds components
/// through `T` until a line break or source exhaustion is reached. The next line
/// number and offset advance only after a complete line identity is produced.
pub struct LineIter<'a, S, T>
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + crate::dissect::SourceDissect<'a, View<'a> = &'a S> + ?Sized,
    T: Terminator<'a, S>,
{
    /// The source sequence.
    pub(crate) target_slice: &'a S,

    /// An offset relative to the [`SourceLines`] on its own plane.
    pub(crate) target_offset: usize,

    /// An indice that represents the current line in the source.
    pub(crate) target_index: NonZero<usize>,

    /// A terminator that is used to determine the end of a line.
    pub(crate) terminator: T,
}

impl<'a, S, T> Ord for LineIter<'a, S, T>
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + crate::dissect::SourceDissect<'a, View<'a> = &'a S> + ?Sized,
    T: Terminator<'a, S>,
    S: Ord,
    T: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        let Self {
            target_slice,
            target_offset,
            target_index,
            terminator,
            ..
        } = self;
        let Self {
            target_slice: other_target_slice,
            target_offset: other_target_offset,
            target_index: other_target_index,
            terminator: other_terminator,
            ..
        } = other;

        match target_slice.cmp(other_target_slice) {
            Ordering::Equal => match target_offset.cmp(other_target_offset) {
                Ordering::Equal => match target_index.cmp(other_target_index) {
                    Ordering::Equal => terminator.cmp(other_terminator),
                    target_order => target_order,
                },
                target_order => target_order,
            },
            target_order => target_order,
        }
    }
}

impl<'a, S, T> PartialOrd for LineIter<'a, S, T>
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + crate::dissect::SourceDissect<'a, View<'a> = &'a S> + ?Sized,
    T: Terminator<'a, S>,
    S: PartialOrd,
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let Self {
            target_slice,
            target_offset,
            target_index,
            terminator,
            ..
        } = self;
        let Self {
            target_slice: other_target_slice,
            target_offset: other_target_offset,
            target_index: other_target_index,
            terminator: other_terminator,
            ..
        } = other;

        match target_slice.partial_cmp(other_target_slice) {
            Some(Ordering::Equal) => match target_offset.partial_cmp(other_target_offset) {
                Some(Ordering::Equal) => match target_index.partial_cmp(other_target_index) {
                    Some(Ordering::Equal) => terminator.partial_cmp(other_terminator),
                    target_order => target_order,
                },
                target_order => target_order,
            },
            target_order => target_order,
        }
    }
}

impl<'a, S, T> Eq for LineIter<'a, S, T>
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + crate::dissect::SourceDissect<'a, View<'a> = &'a S> + ?Sized,
    T: Terminator<'a, S>,
    S: Eq,
    T: Eq,
{
}

impl<'a, S, T> PartialEq for LineIter<'a, S, T>
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + crate::dissect::SourceDissect<'a, View<'a> = &'a S> + ?Sized,
    T: Terminator<'a, S>,
    S: PartialEq,
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self {
            target_slice,
            target_offset,
            target_index,
            terminator,
            ..
        } = self;
        let Self {
            target_slice: other_target_slice,
            target_offset: other_target_offset,
            target_index: other_target_index,
            terminator: other_terminator,
            ..
        } = other;

        target_slice == other_target_slice
            && target_offset == other_target_offset
            && target_index == other_target_index
            && terminator == other_terminator
    }
}

impl<'a, S, T> fmt::Debug for LineIter<'a, S, T>
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + crate::dissect::SourceDissect<'a, View<'a> = &'a S> + ?Sized,
    T: Terminator<'a, S>,
    S: fmt::Debug,
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            target_slice,
            target_offset,
            target_index,
            terminator,
            ..
        } = self;

        f.debug_struct("LineIter")
            .field("target_slice", target_slice)
            .field("target_offset", target_offset)
            .field("target_index", target_index)
            .field("terminator", terminator)
            .finish()
    }
}

impl<'a, S, T> Iterator for LineIter<'a, S, T>
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + crate::dissect::SourceDissect<'a, View<'a> = &'a S> + ?Sized,
    T: Terminator<'a, S>,
{
    type Item = LineId<'a, S>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let &mut Self {
            target_slice,
            target_offset,
            target_index: line_index,
            ref mut terminator,
        } = self;

        let target_view = target_slice.offset(target_offset)?;

        let line_id: Option<LineId<'_, S>> = {
            let mut target_iter = target_view.enumerate();

            loop {
                if let Some((component_index, target_component)) = target_iter.next() {
                    let ctx = TerminatorContext::from_raw_parts(
                        target_slice,
                        target_component,
                        Span::new(target_offset + component_index, target_component.size()),
                    );

                    let outcome = terminator.state(&ctx);

                    match outcome {
                        Outcome::Continue(_) => continue,
                        Outcome::Break(target_break) => {
                            let sequence_end = component_index + target_component.size().get();

                            let line_length = NonZero::new(sequence_end.saturating_sub(target_break.span().length().get()));

                            let line_span = line_length.map(|length| Span::new(target_offset, length));

                            let line_id = if let Some(line_span) = line_span {
                                let target_line = target_slice.dissect(line_span);

                                let line_content = LineContent::from_raw_parts(target_line, line_span);

                                LineId::full(line_index, line_content, target_break)
                            } else {
                                LineId::break_only(line_index, target_break)
                            };

                            break Some(line_id);
                        }
                    }
                } else {
                    break match target_view.footprint() {
                        Some(target_footprint) => {
                            let line_span = Span::new(target_offset, target_footprint.length());
                            let target_line = target_slice.dissect(line_span);
                            let line_content = LineContent::from_raw_parts(target_line, line_span);

                            Some(LineId::standalone(line_index, line_content))
                        }
                        None => None,
                    };
                }
            }
        };

        match line_id {
            Some(ref line_id) => {
                self.target_offset += line_id.span().length().get();
                self.target_index = line_index.saturating_add(1);
            }
            None => (),
        }

        line_id
    }
}

impl<'a, S, T> LineIter<'a, S, T>
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + crate::dissect::SourceDissect<'a, View<'a> = &'a S> + ?Sized,
    T: Terminator<'a, S>,
{
    /// Create a new [`LineIter`] from a source, starting at the beginning.
    ///
    /// Uses the default terminator for the terminator type.
    #[inline]
    pub fn new(target_source: &'a S) -> Self
    where
        T: Default,
    {
        Self::new_with(target_source, T::default())
    }

    /// Create a new [`LineIter`] from a source, starting at the specified
    /// offset.
    ///
    /// Uses the default terminator for the terminator type.
    #[inline]
    pub fn new_at(target_source: &'a S, target_offset: usize) -> Self
    where
        T: Default,
    {
        Self::new_at_with(target_source, target_offset, T::default())
    }

    /// Create a new [`LineIter`] from a source, starting at the beginning.
    ///
    /// If you wish to start at a specific index, use [`LineIter::new_at`].
    #[inline]
    pub const fn new_with(target_source: &'a S, terminator: T) -> Self {
        let target_slice = target_source;

        let target_offset = 0;

        let target_index = NonZero::<usize>::MIN;

        Self {
            target_slice,
            target_offset,
            target_index,
            terminator,
        }
    }

    /// Create a new [`LineIter`] from a source, starting at a specific offset,
    /// and with the specified [`Terminator`].
    ///
    /// # Remarks
    ///
    /// This will skip any lines that are before the target offset.
    /// If the offset itself is contained within a line, that line will be
    /// truncated.
    ///
    /// For example:
    /// - if the source is `abc\ndef\nghi` and the target offset is `4`, the first line will be `def`.
    /// - if the source is `abc\ndef\nghi` and the target offset is `2`, the first line will be `c`, as offset 3 is part of the `abc\n`
    ///   line.
    #[inline]
    pub const fn new_at_with(target_source: &'a S, target_offset: usize, terminator: T) -> Self {
        let target_slice = target_source;

        let target_index = NonZero::<usize>::MIN;

        Self {
            target_slice,
            target_offset,
            target_index,
            terminator,
        }
    }

    /// Construct a new [`LineIter`] from its individual components:
    ///
    /// - The target source.
    /// - The target offset.
    /// - The target index.
    /// - The terminator to use.
    #[inline]
    pub const fn from_raw_parts(target_slice: &'a S, target_offset: usize, target_index: NonZero<usize>, terminator: T) -> Self {
        Self {
            target_slice,
            target_offset,
            target_index,
            terminator,
        }
    }

    /// Determine the current index of this iterator.
    ///
    /// This is the index that is the next to-be yielded line.
    #[inline]
    pub const fn index(&self) -> NonZero<usize> {
        let &Self { target_index, .. } = self;

        target_index
    }

    /// Determine the relative offset from this iterator's source
    /// and its current position.
    #[inline]
    pub const fn offset(&self) -> usize {
        let &Self { target_offset, .. } = self;

        target_offset
    }

    /// Determine the source of this iterator.
    #[inline]
    pub const fn source(&self) -> &'a S {
        let &Self { target_slice, .. } = self;

        target_slice
    }
}

impl<'a, S, T> Clone for LineIter<'a, S, T>
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + crate::dissect::SourceDissect<'a, View<'a> = &'a S> + ?Sized,
    T: Terminator<'a, S>,
    T: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let &Self {
            target_index,
            target_slice,
            target_offset,
            ref terminator,
            ..
        } = self;

        let terminator = terminator.clone();

        Self {
            target_index,
            target_offset,
            target_slice,
            terminator,
        }
    }
}

impl<'a, S, T> Copy for LineIter<'a, S, T>
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + crate::dissect::SourceDissect<'a, View<'a> = &'a S> + ?Sized,
    T: Terminator<'a, S>,
    T: Copy,
{
}
