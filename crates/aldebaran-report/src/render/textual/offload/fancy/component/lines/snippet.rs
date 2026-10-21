//! Source lines paired with the annotations selected for Fancy output.
//!
//! [`SnippetSegment`] distinguishes a line with relevant annotations from an
//! unannotated line. [`RelevantSegment`] retains borrowed annotations in their
//! caller-supplied order while owning only the collection of references.

use core::fmt;

use aldebaran_baseline::prelude::{Nonempty, RefNonempty};

use aldebaran_source::prelude::{LineId, SourceLines};

use crate::{annotated::Annotations, report::SourceReport};

/// A source line and any report annotations that intersect it.
pub enum SnippetSegment<'source, 'borrow, E>
where
    E: SourceReport<'source>,
    E::Source: SourceLines<'source>,
    'source: 'borrow,
{
    /// A line with matching annotations.
    Relevant(
        /// Source segment together with every annotation intersecting it.
        RelevantSegment<'source, 'borrow, E>,
    ),

    /// A line without matching annotations.
    Irrelevant(
        /// Source line omitted from the annotated excerpt.
        IrrelevantSegment<'source, E>,
    ),
}

impl<'source, 'borrow, E> SnippetSegment<'source, 'borrow, E>
where
    E: SourceReport<'source>,
    E::Source: SourceLines<'source>,
    'source: 'borrow,
{
    /// Determine the [`line id`](LineId) for this snippet segment.
    #[inline]
    pub const fn id(&self) -> &LineId<'source, E::Source> {
        match self {
            Self::Relevant(segment) => segment.id(),
            Self::Irrelevant(segment) => segment.id(),
        }
    }
}

impl<'source, 'borrow, E> fmt::Debug for SnippetSegment<'source, 'borrow, E>
where
    'source: 'borrow,
    E: SourceReport<'source>,
    E::Source: SourceLines<'source>,
    <E::Annotations as Annotations>::Annotation: fmt::Debug,
    <E::Source as SourceLines<'source>>::Line: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Relevant(arg0) => f.debug_tuple("Relevant").field(arg0).finish(),
            Self::Irrelevant(arg0) => f.debug_tuple("Irrelevant").field(arg0).finish(),
        }
    }
}

/// A source line paired with its intersecting annotation references.
// NOTE(invariant): None.
pub struct RelevantSegment<'source, 'borrow, E>
where
    E: SourceReport<'source>,
    E::Source: SourceLines<'source>,
{
    /// The line identifier for this snippet segment.
    pub(in crate::render::textual::offload::fancy::component::lines) line_id: LineId<'source, E::Source>,

    /// Caller-ordered references to the report annotations selected for this line.
    pub(in crate::render::textual::offload::fancy::component::lines) annotations:
        Nonempty<&'borrow <E::Annotations as Annotations>::Annotation>,
}

impl<'source, 'borrow, E> fmt::Debug for RelevantSegment<'source, 'borrow, E>
where
    E: SourceReport<'source>,
    E::Source: SourceLines<'source>,
    'source: 'borrow,
    <E::Annotations as Annotations>::Annotation: fmt::Debug,
    <E::Source as SourceLines<'source>>::Line: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { line_id, annotations } = self;

        f.debug_struct("RelevantSegment")
            .field("line_id", line_id)
            .field("annotations", annotations)
            .finish()
    }
}

impl<'source, 'borrow, E> RelevantSegment<'source, 'borrow, E>
where
    E: SourceReport<'source>,
    E::Source: SourceLines<'source>,
    'source: 'borrow,
{
    /// Retrieve the [`line identifier`](LineId) for this snippet segment.
    #[inline]
    pub const fn id(&self) -> &LineId<'source, E::Source> {
        let &Self { ref line_id, .. } = self;

        line_id
    }

    /// Caller-ordered annotations, borrowing the reference collection for this view.
    #[inline]
    pub const fn annotations(&self) -> RefNonempty<'_, &'borrow <E::Annotations as Annotations>::Annotation> {
        let &Self { ref annotations, .. } = self;

        annotations.as_ref()
    }
}

/// A source line with no annotation intersecting its span.
// NOTE(invariant): None.
pub struct IrrelevantSegment<'a, E>(
    /// Line identifier retained for an unannotated source segment.
    pub(in crate::render::textual::offload::fancy::component::lines) LineId<'a, E::Source>,
)
where
    E: SourceReport<'a>,
    E::Source: SourceLines<'a>;

impl<'a, E> IrrelevantSegment<'a, E>
where
    E: SourceReport<'a>,
    E::Source: SourceLines<'a>,
{
    /// Retrieve the [`line identifier`](LineId) for this snippet segment.
    pub const fn id(&self) -> &LineId<'a, E::Source> {
        let &Self(ref line_id) = self;

        line_id
    }
}

impl<'a, E> fmt::Debug for IrrelevantSegment<'a, E>
where
    E: SourceReport<'a>,
    E::Source: SourceLines<'a>,
    <E::Source as SourceLines<'a>>::Line: fmt::Debug,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self(line_id) = self;

        f.debug_struct("IrrelevantSegment").field("line_id", line_id).finish()
    }
}
