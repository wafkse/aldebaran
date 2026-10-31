//! Output rows within a source excerpt.
//!
//! A long source line can occupy several [`Phase`] values, which share the
//! rendering context and original source coordinates.

use core::{fmt, num::NonZero, ops::Range};

use alloc::vec::Vec;

use aldebaran_ansi::prelude::Style;
use aldebaran_baseline::prelude::Span;
use aldebaran_print::{
    layout::TextRow,
    prelude::{Combine, Print},
};

use crate::annotated::{Annotated, title::Title};
use crate::render::textual::offload::fancy::Theme;

use super::{
    Context, Gutter,
    part::{AnnotationMessage, AnnotationReference, SourcePart, Underline, UnderlineRow, UnderlineSegment},
};

/// A row of source context and the annotations displayed beneath it.
///
/// Source coordinates are retained. Text and markers, however, align in terminal
/// cells after the gutter.
///
/// # Remarks
///
/// A phase can also represent an EOF insertion without stored source content.
// NOTE(invariant): Text.CoordinateSpace = Markers.CoordinateSpace && (LineNumber.IsSome => LineNumber.Digits <= Gutter.LineNumberDigitWidth)
pub(in crate::render::textual::offload::fancy::detail) struct Phase<'gutter> {
    /// Content shown at this source location.
    row_content: PhaseContent<'gutter>,

    /// Source line number, absent on continuation rows.
    line_number: Option<NonZero<usize>>,

    /// Alignment with the other rows of the excerpt.
    gutter: &'gutter Gutter,
}

/// Content shown by a source row, including an insertion with no stored text.
enum PhaseContent<'layout> {
    /// A wrapped portion of a physical source line.
    Content {
        /// Exclusive source endpoint of the line's content, before its break.
        content_end: usize,

        /// Borrowed displayed text and fixed-size terminal cells for this row.
        text_row: TextRow<'layout>,
    },

    /// The insertion position after a final line break.
    Terminal {
        /// Source coordinate of the insertion.
        insertion_offset: usize,
    },
}

impl<'gutter> Phase<'gutter> {
    /// Present a borrowed source row under the excerpt's gutter.
    #[inline]
    pub(in crate::render::textual::offload::fancy::detail) const fn content(
        text_row: TextRow<'gutter>,
        content_end: usize,
        line_number: Option<NonZero<usize>>,
        gutter: &'gutter Gutter,
    ) -> Self {
        let row_content = PhaseContent::Content { content_end, text_row };

        Self {
            row_content,
            line_number,
            gutter,
        }
    }

    /// Create source context for an insertion after a final line break.
    #[inline]
    pub(in crate::render::textual::offload::fancy::detail) const fn terminal(
        insertion_offset: usize,
        line_number: NonZero<usize>,
        gutter: &'gutter Gutter,
    ) -> Self {
        let row_content = PhaseContent::Terminal { insertion_offset };
        let line_number = Some(line_number);

        Self {
            row_content,
            line_number,
            gutter,
        }
    }

    /// Write the source row, numbering only the first row of a wrapped line.
    #[inline]
    pub(in crate::render::textual::offload::fancy::detail) fn line<W, T, I>(&self, context: &mut Context<'_, '_, W, T, I>) -> fmt::Result
    where
        W: fmt::Write,
        T: Theme,
    {
        let Self {
            row_content,
            line_number,
            gutter,
        } = self;

        let source = match row_content {
            PhaseContent::Content { text_row, .. } => Some(SourcePart(text_row.text())),
            PhaseContent::Terminal { .. } => None,
        };

        gutter
            .prefix(*line_number, context.fancy_settings)
            .sequence(source)
            .sequence('\n')
            .print(context.sink)
    }

    /// Prepare row geometry once, assign lanes, then print borrowed marker rows.
    pub(in crate::render::textual::offload::fancy::detail) fn marker<A, W, T, I>(
        &self,
        annotation_record_list: &mut [AnnotationRecord<'_, A>],
        annotation_index_list: &[usize],
        marker_layout: &mut MarkerLayout,
        context: &mut Context<'_, '_, W, T, I>,
    ) -> fmt::Result
    where
        A: Annotated,
        W: fmt::Write,
        T: Theme,
        I: Iterator<Item = Style>,
    {
        let Self { gutter, .. } = self;

        let MarkerLayout {
            projection_list,
            underline_segment_list,
            lane_tree,
        } = marker_layout;

        projection_list.clear();
        underline_segment_list.clear();

        for &annotation_index in annotation_index_list {
            let annotation_record = annotation_record_list.get_mut(annotation_index).ok_or(fmt::Error)?;

            let AnnotationRecord {
                target_span,
                has_visible_projection,
                ..
            } = annotation_record;

            let column_range = self.marker_columns(*target_span);

            *has_visible_projection = column_range.is_some();

            if let Some(column_range) = column_range {
                projection_list.push(Projection {
                    annotation_index,
                    column_range,
                    lane_index: usize::MIN,
                    endpoint_order_index: usize::MIN,
                });
            }
        }

        if let usize::MIN = projection_list.len() {
            Ok(())
        } else {
            projection_list.sort_unstable_by_key(
                |Projection {
                     column_range,
                     annotation_index,
                     ..
                 }| (column_range.end, *annotation_index),
            );
            lane_tree.reset(projection_list.len());

            for (
                position,
                Projection {
                    column_range,
                    lane_index,
                    endpoint_order_index,
                    ..
                },
            ) in projection_list.iter_mut().enumerate()
            {
                *lane_index = lane_tree.assign(column_range.start, column_range.end);
                *endpoint_order_index = position;
            }

            projection_list.sort_unstable_by_key(
                |Projection {
                     lane_index,
                     endpoint_order_index,
                     ..
                 }| (*lane_index, *endpoint_order_index),
            );

            for Projection {
                annotation_index,
                column_range,
                ..
            } in projection_list.iter()
            {
                let annotation_record = annotation_record_list.get_mut(*annotation_index).ok_or(fmt::Error)?;

                let AnnotationRecord {
                    target_span,
                    marker_presentation,
                    ..
                } = annotation_record;

                let (marker, style) = *marker_presentation.get_or_insert_with(|| context.marker(*target_span));

                let width = u32::try_from(column_range.end - column_range.start).unwrap_or(u32::MAX);
                let underline = Underline { marker, style, width };

                let column_range = column_range.clone();

                underline_segment_list.push(UnderlineSegment { column_range, underline });
            }

            let mut segment_start_index = usize::MIN;

            for lane_projection_list in projection_list.chunk_by(|left, right| {
                let Projection {
                    lane_index: left_lane_index,
                    ..
                } = left;

                let Projection {
                    lane_index: right_lane_index,
                    ..
                } = right;

                left_lane_index == right_lane_index
            }) {
                let segment_end_index = segment_start_index + lane_projection_list.len();
                let underline_segment_list = underline_segment_list
                    .get(segment_start_index..segment_end_index)
                    .ok_or(fmt::Error)?;
                let colored = context.fancy_settings.colored_ref();
                let underline_row = UnderlineRow {
                    underline_segment_list,
                    colored,
                };

                gutter
                    .prefix(None, context.fancy_settings)
                    .sequence(underline_row)
                    .sequence('\n')
                    .print(context.sink)?;
                segment_start_index = segment_end_index;
            }

            Ok(())
        }
    }

    /// Emit a selected message using the row projection and cached marker identity.
    pub(in crate::render::textual::offload::fancy::detail) fn message<A, W, T, I>(
        &self,
        annotation_record: &mut AnnotationRecord<'_, A>,
        context: &mut Context<'_, '_, W, T, I>,
    ) -> fmt::Result
    where
        A: Annotated,
        <A::Title as Title>::Context: Default,
        W: fmt::Write,
        T: Theme,
    {
        let Self { row_content, gutter, .. } = self;

        let AnnotationRecord {
            annotation,
            target_span,
            marker_presentation,
            has_visible_projection,
            message_emitted,
            ..
        } = annotation_record;

        if *message_emitted || !*has_visible_projection {
            Ok(())
        } else {
            let (should_emit_message, emphasized) = match row_content {
                PhaseContent::Content { text_row, content_end } => {
                    let row_end = text_row.source_range().end;
                    let should_emit_message = target_span.end() <= row_end || row_end == *content_end;

                    (should_emit_message, true)
                }
                PhaseContent::Terminal { .. } => (true, false),
            };

            if should_emit_message {
                let (marker, style) = marker_presentation.ok_or(fmt::Error)?;

                let colored = context.fancy_settings.colored_ref();
                let reference = AnnotationReference {
                    marker,
                    style,
                    colored,
                    emphasized,
                };
                let annotation = *annotation;
                let message = AnnotationMessage { annotation, reference };

                gutter
                    .prefix(None, context.fancy_settings)
                    .sequence("  ")
                    .sequence(message)
                    .sequence('\n')
                    .print(context.sink)?;

                if emphasized {
                    gutter.prefix(None, context.fancy_settings).sequence('\n').print(context.sink)?;
                }

                *message_emitted = true;
            }

            Ok(())
        }
    }

    /// Terminal columns to mark for a source target or the final insertion.
    #[inline]
    pub(in crate::render::textual::offload::fancy::detail) fn marker_columns(&self, target_span: Span) -> Option<Range<usize>> {
        let Self { row_content, .. } = self;

        match row_content {
            PhaseContent::Content { text_row, content_end } => {
                let is_content_end_insertion = target_span == Span::unit(*content_end);
                let is_final_content_row = text_row.source_range().end == *content_end;

                if is_content_end_insertion && is_final_content_row {
                    let insertion_column = text_row.cells().len();

                    Some(insertion_column..insertion_column + 1)
                } else {
                    text_row
                        .columns_for(target_span.range())
                        .map(|column_range| column_range.start..column_range.end.max(column_range.start + 1))
                }
            }
            PhaseContent::Terminal { insertion_offset } => (target_span == Span::unit(*insertion_offset)).then_some(usize::MIN..1),
        }
    }
}

/// One annotation retained once for the whole excerpt.
// NOTE(invariant): MarkerPresentation.IsSome => MarkerPresentation = AssignedPresentation(TargetSpan)
pub(in crate::render::textual::offload::fancy::detail) struct AnnotationRecord<'a, A> {
    /// Original report annotation.
    pub(in crate::render::textual::offload::fancy::detail) annotation: &'a A,

    /// Cached original target coordinates.
    pub(in crate::render::textual::offload::fancy::detail) target_span: Span,

    /// Last physical line whose content can project the target.
    pub(in crate::render::textual::offload::fancy::detail) last_line_number: Option<NonZero<usize>>,

    /// Marker identity assigned on its first visible row.
    pub(in crate::render::textual::offload::fancy::detail) marker_presentation: Option<(char, Style)>,

    /// Whether the current row has a visible projection.
    pub(in crate::render::textual::offload::fancy::detail) has_visible_projection: bool,

    /// Whether its annotation message has already been written.
    pub(in crate::render::textual::offload::fancy::detail) message_emitted: bool,
}

/// An annotation's projected columns and stable lane ordering.
// NOTE(invariant): ColumnRange.Start < ColumnRange.End
struct Projection {
    /// Index into the excerpt's annotation table.
    annotation_index: usize,

    /// Terminal cells selected on the current row.
    column_range: Range<usize>,

    /// Assigned output lane.
    lane_index: usize,

    /// Position in stable endpoint order.
    endpoint_order_index: usize,
}

/// Scratch storage retained across every physical and wrapped row.
#[derive(Default)]
// NOTE(invariant): None.
pub(in crate::render::textual::offload::fancy::detail) struct MarkerLayout {
    /// Temporary row projections, sorted without allocating sort storage.
    projection_list: Vec<Projection>,

    /// Prepared marker runs in lane order.
    underline_segment_list: Vec<UnderlineSegment>,

    /// First-compatible-lane search tree.
    lane_tree: LaneTree,
}

/// Minimum lane endpoints supporting first-compatible placement in logarithmic time.
#[derive(Default)]
// NOTE(invariant): LaneEndpointList.IsEmpty Or (LaneEndpointList.Length = 2 * LeafStartIndex And IsPowerOfTwo(LeafStartIndex))
struct LaneTree {
    /// Minimum occupied endpoint for each subtree; unused lanes end at zero.
    lane_endpoint_list: Vec<usize>,

    /// Index of the first leaf and maximum number of lanes.
    leaf_start_index: usize,
}

impl LaneTree {
    /// Clear lane endpoints while retaining storage from previous rows.
    pub(in crate::render::textual::offload::fancy::detail) fn reset(&mut self, lane_count: usize) {
        let Self {
            lane_endpoint_list,
            leaf_start_index,
        } = self;

        *leaf_start_index = lane_count.next_power_of_two();
        lane_endpoint_list.resize(*leaf_start_index * 2, usize::MIN);
        lane_endpoint_list.fill(usize::MIN);
    }

    /// Select the lowest numbered compatible lane, preserving the old greedy order.
    pub(in crate::render::textual::offload::fancy::detail) fn assign(&mut self, start_column: usize, end_column: usize) -> usize {
        let Self {
            lane_endpoint_list,
            leaf_start_index,
        } = self;

        let mut node_index = 1;

        while node_index < *leaf_start_index {
            let left_child_index = node_index * 2;

            assert!(left_child_index + 1 < lane_endpoint_list.len(), "lane tree children exist");

            node_index = if lane_endpoint_list[left_child_index] <= start_column {
                left_child_index
            } else {
                left_child_index + 1
            };
        }

        let lane_index = node_index - *leaf_start_index;

        assert!(node_index < lane_endpoint_list.len(), "selected lane exists");

        lane_endpoint_list[node_index] = end_column;

        while node_index > 1 {
            node_index /= 2;

            let left_child_index = node_index * 2;

            assert!(left_child_index + 1 < lane_endpoint_list.len(), "lane tree children exist");

            lane_endpoint_list[node_index] = lane_endpoint_list[left_child_index].min(lane_endpoint_list[left_child_index + 1]);
        }

        lane_index
    }
}

#[cfg(test)]
mod tests {
    //! Tests module for the [`phase`] module.
    //!
    //! [`phase`]: super

    use alloc::vec::Vec;

    use super::LaneTree;

    #[test]
    fn logarithmic_lane_assignment_matches_repeated_greedy_passes() {
        let mut tree = LaneTree::default();

        for count in [1, 16, 257, 32] {
            let mut indexed_range_list: Vec<_> = (usize::MIN..count)
                .map(|index| {
                    let start = index * 73 % 97;

                    (start..start + 1 + index * 19 % 100, index)
                })
                .collect();

            indexed_range_list.sort_unstable_by_key(|(range, index)| (range.end, *index));

            let mut pending_range_list: Vec<_> = indexed_range_list.iter().enumerate().collect();
            let mut expected_lane_list = alloc::vec![usize::MIN; count];
            let mut lane = usize::MIN;

            while let Some((_, (first, _))) = pending_range_list.first() {
                let mut rightmost = first.start;

                pending_range_list.retain(|(position, (range, _))| {
                    if range.start < rightmost {
                        return true;
                    }

                    assert!(*position < expected_lane_list.len(), "original position exists");

                    expected_lane_list[*position] = lane;
                    rightmost = range.end;

                    false
                });

                lane += 1;
            }

            tree.reset(count);

            let actual_lane_list: Vec<_> = indexed_range_list
                .iter()
                .map(|(range, _)| tree.assign(range.start, range.end))
                .collect();

            assert_eq!(
                actual_lane_list, expected_lane_list,
                "lane and marker encounter order must remain stable"
            );
        }
    }
}
