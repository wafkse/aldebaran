//! This module renders a diagnostic's heading and annotated source with the same
//! presentation settings.
//!
//! See [`Context`] and [`Excerpt`] for further information.

mod part;
mod phase;

use core::{fmt, mem, num::NonZero, ops::RangeInclusive};

use alloc::vec::Vec;

use aldebaran_ansi::prelude::Style;
use aldebaran_baseline::prelude::{Span, Spanned};
use aldebaran_print::{
    layout::{LayoutOptions, SourceText, TextLayout},
    prelude::{Combine, Print},
    visualize::Visualize,
};
use aldebaran_source::prelude::{
    Component, LineExtent, LineSegmented, Location, Source, SourceDissect, SourceIter, SourceLines, SourceMetadata, Terminator,
};
use hashbrown::HashMap;

use crate::{
    annotated::{Annotated, Annotations},
    prelude::{ErrorKind, Title},
    render::textual::{OffloadContext, Present, viewport::Viewport},
    report::{Report, SourceReport, connotate::Connotate},
};

use self::{
    part::{ExcerptBoundary, GutterLine, Header, SourceLocation},
    phase::{AnnotationRecord, MarkerLayout, Phase},
};
use super::{
    Colored, Fancy, FancySettings, LayoutSettings, Theme,
    charset::indicate::Marker,
    layout::{SourceSpacing, padding::HorizontalPadding},
};

/// The annotation type supplied by a report.
type Annotation<E> = <<E as Report>::Annotations as Annotations>::Annotation;

/// State for rendering a diagnostic consistently across its output rows.
///
/// Operations use the same settings and marker assignments, so an annotation
/// remains recognizable when it wraps or spans source lines.
// NOTE(invariant): ForAll(Span In Assigned): Assigned[Span] = FirstAssignment(Span, ThisRender)
struct Context<'a, 'b, W, T: Theme, I> {
    /// Destination for this diagnostic's output.
    sink: &'b mut W,

    /// Presentation settings shared by the heading and source excerpt.
    fancy_settings: &'a FancySettings<T>,

    /// Available display area, measured in terminal cells.
    viewport: &'a Viewport,

    /// Glyphs available for annotations without an assigned marker.
    marker_iter: Marker<'static>,

    /// Styles available for annotations without an assigned marker.
    style_iter: I,

    /// Style for annotations after the theme's palette is exhausted.
    fallback: Style,

    /// Marker identities keyed by original source span.
    assigned: HashMap<Span, (char, Style)>,
}

impl<W, T: Theme, I: Iterator<Item = Style>> Context<'_, '_, W, T, I> {
    /// A span keeps its marker across source rows and messages.
    fn marker(&mut self, target_span: Span) -> (char, Style) {
        let &mut Self {
            ref mut marker_iter,
            ref mut style_iter,
            fallback,
            ref mut assigned,
            ..
        } = self;

        *assigned.entry(target_span).or_insert_with(|| {
            let marker = marker_iter.next().unwrap_or(marker_iter.default());
            let style = style_iter.next().unwrap_or(fallback);

            (marker, style)
        })
    }
}

/// A borrowed excerpt that shares one annotation table across its source lines.
// NOTE(invariant): (LineNumberRange, TerminalLineNumber) = Describe(Selection(Source, Extents))
struct Excerpt<'source, 'extent, E, Te>
where
    E: SourceReport<'source>,
    Te: Terminator<'source, E::Source>,
{
    /// Original source backing every row.
    source: &'source E::Source,

    /// Repeatable line traversal without collected line identities.
    line_extents: &'extent <E::Source as SourceLines<'source>>::Extents<Te>,

    /// Original annotations, cached presentation, and final message locations.
    annotation_record_list: Vec<AnnotationRecord<'source, Annotation<E>>>,

    /// Inclusive original line-number range.
    line_number_range: RangeInclusive<NonZero<usize>>,

    /// Synthetic final row for an insertion after a terminating break.
    terminal_line_number: Option<NonZero<usize>>,

    /// Shared gutter geometry.
    gutter: Gutter,
}

impl<'source, 'extent, E, Te> Excerpt<'source, 'extent, E, Te>
where
    E: SourceReport<'source>,
    Te: Terminator<'source, E::Source>,
{
    /// Prepare message locations in one traversal, retaining no source-line list.
    fn new<W, T, I>(
        report: &'source E,
        line_extents: &'extent <E::Source as SourceLines<'source>>::Extents<Te>,
        context: &Context<'_, '_, W, T, I>,
    ) -> Self
    where
        T: Theme,
    {
        let source = report.source();
        let source_end = source.size();
        let mut annotation_record_list: Vec<_> = report
            .annotations()
            .list()
            .into_iter()
            .flat_map(|annotation_list| annotation_list.iter())
            .map(|annotation| {
                let target_span = annotation.target();

                AnnotationRecord {
                    annotation,
                    target_span,
                    last_line_number: None,
                    marker_presentation: None,
                    has_visible_projection: false,
                    message_emitted: false,
                }
            })
            .collect();
        let has_eof = annotation_record_list
            .iter()
            .any(|AnnotationRecord { target_span, .. }| *target_span == Span::unit(source_end));
        let mut terminal_line_number = None;

        for line_id in line_extents.lines() {
            if has_eof && line_id.span().end() == source_end && line_id.content_break().is_some() {
                terminal_line_number = line_id.number().checked_add(1);
            }

            if let Some(line_content) = line_id.content() {
                let span = line_content.span();

                for AnnotationRecord {
                    target_span,
                    last_line_number,
                    ..
                } in &mut annotation_record_list
                {
                    if target_span.overlaps(span) || *target_span == Span::unit(span.end()) {
                        *last_line_number = Some(line_id.number());
                    }
                }
            }
        }

        let selected_line_range = line_extents.metadata().lines();
        let line_number_range = selected_line_range.start()..=selected_line_range.end();
        let line_number_digit_width = terminal_line_number.unwrap_or(*line_number_range.end()).get().ilog10() + 1;
        let fancy_settings = context.fancy_settings;
        let layout_settings = fancy_settings
            .layout()
            .unwrap_or_else(|| LayoutSettings::viewport(context.viewport));
        let line_column_padding = layout_settings.line_column_padding();
        let source_spacing = layout_settings.source_spacing();
        let gutter = Gutter {
            line_number_digit_width,
            line_column_padding,
            source_spacing,
        };

        Self {
            source,
            line_extents,
            annotation_record_list,
            line_number_range,
            terminal_line_number,
            gutter,
        }
    }

    /// Render through reusable source, row, annotation, and lane scratch buffers.
    fn render<W, T, I>(self, context: &mut Context<'_, '_, W, T, I>) -> fmt::Result
    where
        W: fmt::Write,
        T: Theme,
        I: Iterator<Item = Style>,
        <E::Source as SourceMetadata<'source>>::Metadata: Location,
        <<E::Source as SourceMetadata<'source>>::Metadata as Print>::Context: Default,
        <<<E::Annotations as Annotations>::Annotation as Annotated>::Title as Title>::Context: Default,
        <<E::Source as SourceLines<'source>>::Line as LineSegmented<'source, E::Source>>::View: Visualize,
    {
        let Self {
            source,
            line_extents,
            mut annotation_record_list,
            line_number_range,
            terminal_line_number,
            gutter,
        } = self;

        let source_end = source.size();
        let fancy_settings = context.fancy_settings;
        let metadata = source.metadata();
        let style = fancy_settings.theme().lines();
        let colored = fancy_settings.colored_ref();
        let location = SourceLocation {
            metadata,
            line_number_range: &line_number_range,
            style,
            colored,
        };
        let delimit = fancy_settings.charset().delimit();
        let opening_boundary = ExcerptBoundary::Start {
            gutter: &gutter,
            delimit,
            location,
        };

        opening_boundary.sequence('\n').print(context.sink)?;

        let column_count = NonZero::new(usize::from(context.viewport.extents().width())).ok_or(fmt::Error)?;
        let tab_width = NonZero::<usize>::from(fancy_settings.tab_width());
        let layout_options = LayoutOptions::new(column_count, tab_width);
        let input = SourceText::new(usize::MIN);
        let mut text_layout = TextLayout::new(input, layout_options);
        let mut source_buffer = SourceText::new(usize::MIN);
        let mut annotation_index_list = Vec::new();
        let mut marker_layout = MarkerLayout::default();

        for line_id in line_extents.lines() {
            let span = line_id.span();

            annotation_index_list.clear();
            annotation_index_list.extend(annotation_record_list.iter().enumerate().filter_map(
                |(index, AnnotationRecord { target_span, .. })| {
                    (target_span.overlaps(span) || (*target_span == Span::unit(source_end) && span.end() == source_end)).then_some(index)
                },
            ));

            if !annotation_index_list.is_empty() {
                if let Some(line_content) = line_id.content() {
                    let span = line_content.span();
                    let mut line_number = Some(line_id.number());
                    let text = if let Some(text) = line_content.value().as_str() {
                        SourceText::borrowed(text, span.start())
                    } else {
                        source_buffer = source_buffer.cleared(span.start());

                        let source_view = source.dissect(span);
                        let mut window_start_offset = usize::MIN;

                        for (component_offset, component) in source_view.enumerate() {
                            let window_end_offset = component_offset + component.size().get();
                            let window_width = NonZero::new(window_end_offset - window_start_offset).ok_or(fmt::Error)?;

                            if let Some(component_view) = line_content.value().window(Span::new(window_start_offset, window_width)) {
                                source_buffer.push(window_width, &component_view)?;
                                window_start_offset = window_end_offset;
                            }
                        }

                        mem::replace(&mut source_buffer, SourceText::new(usize::MIN))
                    };

                    let (unused_source_text, mut line_layout) = text_layout.reused(text, layout_options);

                    drop(unused_source_text);

                    while let Some(text_row) = line_layout.next_row() {
                        let phase = Phase::content(text_row, span.end(), line_number.take(), &gutter);

                        phase.line(context)?;
                        phase.marker(&mut annotation_record_list, &annotation_index_list, &mut marker_layout, context)?;

                        for &index in &annotation_index_list {
                            let annotation_record = annotation_record_list.get_mut(index).ok_or(fmt::Error)?;

                            let AnnotationRecord { last_line_number, .. } = annotation_record;

                            if *last_line_number == Some(line_id.number()) {
                                phase.message(annotation_record, context)?;
                            }
                        }
                    }

                    let (previous_source_text, reused_text_layout) = line_layout.reused(SourceText::new(usize::MIN), layout_options);

                    source_buffer = previous_source_text.cleared(usize::MIN);
                    text_layout = reused_text_layout;
                }

                if let Some(line_number) = terminal_line_number
                    && line_id.span().end() == source_end
                {
                    let phase = Phase::terminal(source_end, line_number, &gutter);

                    phase.line(context)?;
                    phase.marker(&mut annotation_record_list, &annotation_index_list, &mut marker_layout, context)?;

                    for &index in &annotation_index_list {
                        let annotation_record = annotation_record_list.get_mut(index).ok_or(fmt::Error)?;

                        phase.message(annotation_record, context)?;
                    }
                }
            }
        }

        let delimit = fancy_settings.charset().delimit();

        ExcerptBoundary::<()>::End { gutter: &gutter, delimit }.print(context.sink)
    }
}

/// Keeps the excerpt's line numbers and frame aligned.
// NOTE(invariant): ForAll(Number In Excerpt.DisplayedNumbers): Digits(Number) <= LineNumberDigitWidth
struct Gutter {
    /// Width of the largest line number, in terminal cells.
    line_number_digit_width: u32,

    /// Space on either side of the line-number column.
    line_column_padding: HorizontalPadding,

    /// Indentation between the separator and source or annotations.
    source_spacing: SourceSpacing,
}

impl Gutter {
    /// Return the terminal columns available before the frame delimiter.
    #[inline]
    const fn width(&self) -> u32 {
        let &Self {
            line_number_digit_width,
            line_column_padding,
            ..
        } = self;

        line_number_digit_width + line_column_padding.absolute()
    }

    /// Build a printable line-number column for this row.
    #[inline]
    pub(in crate::render::textual::offload::fancy::detail) fn line<'a>(
        &'a self,
        line_number: Option<NonZero<usize>>,
        style: Style,
        colored: &'a Colored,
    ) -> GutterLine<'a> {
        GutterLine {
            gutter: self,
            line_number,
            style,
            colored,
        }
    }

    /// Compose the common prefix for source, marker, and message rows.
    #[inline]
    fn prefix<'a, T>(
        &'a self,
        line_number: Option<NonZero<usize>>,
        fancy_settings: &'a FancySettings<T>,
    ) -> impl Print<Context: Default> + use<'a, T>
    where
        T: Theme,
    {
        let &Self { source_spacing, .. } = self;

        self.line(line_number, fancy_settings.theme().lines(), fancy_settings.colored_ref())
            .sequence(fancy_settings.charset().delimit().left())
            .sequence(source_spacing)
    }
}

/// Format a report into the configured destination using its presentation settings.
pub(in crate::render::textual::offload::fancy) fn render<'source, 'input, W, E, T>(
    report: &'source E,
    offload: &mut OffloadContext<'source, 'input, W, Fancy<T>, E>,
) -> fmt::Result
where
    'source: 'input,
    W: fmt::Write,
    E: SourceReport<'source>,
    T: Theme,
    <E::Source as SourceMetadata<'source>>::Metadata: Location,
    <E::Source as SourceLines<'source>>::Line: Print,
    <<E::Source as SourceMetadata<'source>>::Metadata as Print>::Context: Default,
    <<E::Source as SourceLines<'source>>::Line as LineSegmented<'source, E::Source>>::View: Visualize,
    <<E::Kind as ErrorKind>::Descriptor as Print>::Context: Default,
    <E::Title as Title>::Context: Default,
    <<<E::Annotations as Annotations>::Annotation as Annotated>::Title as Title>::Context: Default,
{
    let &mut OffloadContext { ref mut sink, ref present } = offload;

    let &Present {
        input: ref fancy_settings,
        ref viewport,
    } = present;

    let (fallback, style_iter) = fancy_settings.theme().arrow();

    let marker_iter = fancy_settings.charset().indicator().marker();
    let assigned = HashMap::new();
    let mut context = Context {
        sink,
        fancy_settings,
        viewport,
        marker_iter,
        style_iter,
        fallback,
        assigned,
    };
    let kind = report.kind();
    let title = report.title();
    let theme = fancy_settings.theme().metadata(&kind.connotates());
    let colored = fancy_settings.colored_ref();
    let header = Header {
        kind,
        title,
        theme: &theme,
        colored,
    };

    header.sequence('\n').print(context.sink)?;

    match report.target().and_then(|target_span| report.source().segment(target_span)) {
        Some(line_extents) => Excerpt::new(report, &line_extents, &context).render(&mut context),
        None => Ok(()),
    }
}
