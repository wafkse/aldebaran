//! Source snippet segments used by the fancy line renderer.
//!
//! Snippet segments preserve both the visible source view and the annotations
//! associated with relevant intervals. The distinction between relevant and
//! irrelevant pieces lets line rendering retain source gaps without extra scans.

use core::fmt;

use aldebaran_dsa::prelude::{OneOrMore, RefOneOrMore};

use aldebaran_source::prelude::{LineId, SourceLines};

use crate::{annotated::Annotations, report::SourceReport};

/// One visible source segment classified by annotation relevance.
///
/// Relevant segments carry the annotations that intersect the source interval.
/// Irrelevant segments retain the intervening source text so the renderer can
/// reconstruct the complete visible line in one ordered pass.
pub enum SnippetSegment<'source, 'borrow, E>
where
    E: SourceReport<'source>,
    E::Source: SourceLines<'source>,
    'source: 'borrow,
{
    /// A variant that denotes a relevant source segment.
    Relevant(RelevantSegment<'source, 'borrow, E>),

    /// A variant that denotes an irrelevant source segment.
    Irrelevant(IrrelevantSegment<'source, E>),
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

/// Relevant source segment paired with its intersecting annotations.
///
/// The segment owns no report data. It borrows the projected source view and the
/// annotation references selected for that interval during line decomposition.
pub struct RelevantSegment<'source, 'borrow, E>
where
    E: SourceReport<'source>,
    E::Source: SourceLines<'source>,
{
    /// The line identifier for this snippet segment.
    pub(super) line_id: LineId<'source, E::Source>,

    /// The selection type for the annotations regarding this line segment.
    pub(super) annotations: OneOrMore<&'borrow <E::Annotations as Annotations>::Annotation>,
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
        f.debug_struct("RelevantSegment")
            .field("line_id", &self.line_id)
            .field("annotations", &self.annotations)
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

    /// Retrieve the annotations corresponding to this snippet segment.
    #[inline]
    pub fn annotations(&self) -> RefOneOrMore<'borrow, &<E::Annotations as Annotations>::Annotation> {
        let &Self { ref annotations, .. } = self;

        annotations.as_ref()
    }
}

/// A structure that represents an irrelevant segment.
///
/// This is for the line segments that do not contain any annotations and
/// can be safely ignored.
pub struct IrrelevantSegment<'a, E>(pub(super) LineId<'a, E::Source>)
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
        f.debug_struct("IrrelevantSegment").field("line_id", &self.0).finish()
    }
}
