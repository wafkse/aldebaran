//! Line-oriented source rendering for the fancy textual backend.
//!
//! This module projects report annotations onto visible source phases, packs
//! overlapping markers into separate lanes, and emits annotation messages when
//! their final relevant line has been rendered. Terminal source boundaries are
//! preserved as visible insertion markers on the final phase.

mod snippet;

pub use self::snippet::{IrrelevantSegment, RelevantSegment, SnippetSegment};

use core::{fmt, hash, marker, num::NonZero};

use aldebaran_dsa::collect::Vec;

use aldebaran_dsa::prelude::{HashMap, OneOrMore};
use aldebaran_print::prelude::{Combine, Print};

use aldebaran_ansi::{brush::Paintable, predicate::Should, prelude::Style};

use aldebaran_source::prelude::{LineExtent, LineSegmented, Location, Source, SourceLines, SourceMetadata, Terminator};

use aldebaran_span::prelude::{Span, Spanned};

use aldebaran_visualize::visual::Visualize;

use crate::annotated::Annotations;
use crate::{
    prelude::{Annotated, Title},
    render::textual::{
        Present,
        offload::{
            Offload, OffloadContext,
            fancy::{Colored, DefaultTheme, FancySettings, LayoutSettings, Theme},
        },
    },
    report::SourceReport,
};

/// Formatting sink that counts rendered Unicode scalar values.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct VisualWidth(u32);

impl fmt::Write for VisualWidth {
    #[inline]
    fn write_str(&mut self, source: &str) -> fmt::Result {
        let width = u32::try_from(source.chars().count()).unwrap_or(u32::MAX);

        self.0 = self.0.saturating_add(width);

        Ok(())
    }

    #[inline]
    fn write_char(&mut self, _: char) -> fmt::Result {
        self.0 = self.0.saturating_add(1);

        Ok(())
    }
}

/// Line-by-line rendering stage for the fancy textual report.
///
/// This stage selects relevant source lines, splits wide lines into viewport
/// phases, assigns marker lanes to overlapping annotations, and emits each
/// annotation message after its final relevant line. The type carries only the
/// compile-time source, terminator, and theme relationships needed by offloading.
pub struct ByLines<'a, E, Te, Th = DefaultTheme>
where
    E: SourceReport<'a>,
    Th: Theme,
    Te: Terminator<'a, E::Source>,
    E::Source: SourceLines<'a> + SourceMetadata<'a> + 'a,
    <E::Source as SourceMetadata<'a>>::Metadata: Location,
{
    _marker: (
        marker::PhantomData<&'a ()>,
        marker::PhantomData<E>,
        marker::PhantomData<Te>,
        marker::PhantomData<Th>,
    ),
}

impl<'a, E, Th, Te> Copy for ByLines<'a, E, Te, Th>
where
    E: SourceReport<'a>,
    Th: Theme,
    Te: Terminator<'a, E::Source>,
    E::Source: SourceLines<'a> + SourceMetadata<'a> + 'a,
    <E::Source as SourceMetadata<'a>>::Metadata: Location,
{
}

impl<'a, E, Th, Te> Clone for ByLines<'a, E, Te, Th>
where
    E: SourceReport<'a>,
    Th: Theme,
    Te: Terminator<'a, E::Source>,
    E::Source: SourceLines<'a> + SourceMetadata<'a> + 'a,
    <E::Source as SourceMetadata<'a>>::Metadata: Location,
{
    #[inline]
    fn clone(&self) -> Self {
        let &Self { _marker } = self;

        Self { _marker }
    }
}

impl<'a, E, Th, Te> hash::Hash for ByLines<'a, E, Te, Th>
where
    E: SourceReport<'a>,
    Th: Theme,
    Te: Terminator<'a, E::Source>,
    E::Source: SourceLines<'a> + SourceMetadata<'a> + 'a,
    <E::Source as SourceMetadata<'a>>::Metadata: Location,
{
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { _marker, .. } = self;

        _marker.hash(state);
    }
}

impl<'a, E, Th, Te> fmt::Debug for ByLines<'a, E, Te, Th>
where
    E: SourceReport<'a>,
    Th: Theme,
    Te: Terminator<'a, E::Source>,
    E::Source: SourceLines<'a> + SourceMetadata<'a> + 'a,
    <E::Source as SourceMetadata<'a>>::Metadata: Location,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { _marker, .. } = self;

        f.debug_tuple("Lines").finish_non_exhaustive()
    }
}

impl<'a, E, Th, Te> ByLines<'a, E, Te, Th>
where
    E: SourceReport<'a>,
    Th: Theme,
    Te: Terminator<'a, E::Source>,
    E::Source: SourceLines<'a> + SourceMetadata<'a> + 'a,
    <E::Source as SourceMetadata<'a>>::Metadata: Location,
{
    /// Determine whether one source span intersects a rendered source line.
    #[inline]
    fn line_relevant(target: Span, line_span: Span, source_size: usize) -> bool {
        if target.overlaps(line_span) {
            true
        } else if target.start() == source_size {
            line_span.end() == source_size
        } else {
            false
        }
    }

    /// Project one source span onto visible content.
    ///
    /// A unit span beginning at the source length is preserved on the final
    /// visible phase so EOF diagnostics retain their boundary marker.
    #[inline]
    fn phase_target(target: Span, phase_span: Span, content_span: Span) -> Option<Span> {
        match target.intersect(phase_span) {
            Some(span) => Some(span),
            None => {
                let boundary = target.start() == content_span.end();
                let final_phase = phase_span.end() == content_span.end();

                match (boundary, final_phase) {
                    (true, true) => Some(target),
                    _ => None,
                }
            }
        }
    }

    /// Retrieve the right display boundary used for marker lane packing.
    #[inline]
    const fn display_end(target: Span) -> usize {
        target.end()
    }

    /// Retrieve the left display boundary used for marker placement.
    #[inline]
    const fn display_start(target: Span) -> usize {
        target.start()
    }

    /// Count rendered scalar cells in one source interval.
    #[inline]
    fn display_width(line: &<E::Source as SourceLines<'a>>::Line, content_span: Span, start: usize, end: usize) -> u32
    where
        <<E::Source as SourceLines<'a>>::Line as LineSegmented<'a, E::Source>>::View: Visualize,
    {
        let start = start.max(content_span.start());
        let end = end.min(content_span.end());

        if start >= end {
            return 0;
        }

        let Some(length) = NonZero::new(end - start) else {
            return 0;
        };
        let relative = Span::new(start - content_span.start(), length);
        let Some(view) = line.window(relative) else {
            return u32::try_from(end - start).unwrap_or(u32::MAX);
        };
        let mut width = VisualWidth::default();
        let _ = view.visualize(&mut width);

        width.0
    }

    /// Retrieve the number of marker cells occupied by one projected target.
    #[inline]
    fn marker_width(line: &<E::Source as SourceLines<'a>>::Line, content_span: Span, target: Span) -> u32
    where
        <<E::Source as SourceLines<'a>>::Line as LineSegmented<'a, E::Source>>::View: Visualize,
    {
        if target.start() >= content_span.end() {
            1
        } else {
            Self::display_width(line, content_span, target.start(), target.end()).max(1)
        }
    }
}

impl<'source, E, Th, Te> Offload<'source, E> for ByLines<'source, E, Te, Th>
where
    E: SourceReport<'source>,
    Th: Theme + 'static,
    Te: Terminator<'source, E::Source>,
    E::Source: SourceLines<'source> + SourceMetadata<'source>,
    <E::Source as SourceMetadata<'source>>::Metadata: Location,
    <E::Source as SourceLines<'source>>::Line: Print,
    <<E::Source as SourceMetadata<'source>>::Metadata as Print>::Context: Default,
    <<<E::Annotations as Annotations>::Annotation as Annotated>::Title as Title>::Context: Default,
    <<E::Source as SourceLines<'source>>::Line as LineSegmented<'source, E::Source>>::View: Visualize,
{
    type Input<'input>
        = (&'input FancySettings<Th>, <E::Source as SourceLines<'source>>::Extents<Te>)
    where
        'source: 'input;

    type Output = ();

    type Error = fmt::Error;

    fn offload<'input, W>(error: &'source E, ctx: &mut OffloadContext<'source, 'input, W, Self, E>) -> Result<Self::Output, Self::Error>
    where
        'source: 'input,
        W: fmt::Write,
    {
        let &mut OffloadContext { ref mut sink, ref present } = ctx;

        let &Present {
            input: (settings, ref extents),
            viewport,
        } = present;

        let ref is_colored = Should::tuple((settings.colored_ref(), Colored::colored));

        let theme = settings.theme();

        let layout = settings.layout().unwrap_or_else(|| LayoutSettings::viewport(&viewport));

        let charset = settings.charset();

        let metadata = extents.metadata();

        let line_range = metadata.lines();

        let line_no_width = line_range
            .start()
            .get()
            .abs_diff(line_range.end().get())
            .checked_ilog10()
            .unwrap_or(0)
            + 1;

        let line_col_padding = layout.line_column_padding();

        let line_col_width = line_no_width + line_col_padding.absolute();

        ' '.times(line_col_width)
            .sequence(charset.delimit().corner().top_left())
            .sequence(charset.delimit().top())
            .sequence(
                '['.sequence(error.source().metadata())
                    .sequence(
                        ':'.sequence(line_range.start().stylable().styled(theme.lines()).only_when(is_colored))
                            .sequence(':')
                            .sequence(line_range.end().stylable().styled(theme.lines()).only_when(is_colored)),
                    )
                    .sequence(']'),
            )
            .sequence('\n')
            .print(sink)?;

        let source_size = error.source().size();

        let line_snippets: Vec<_> = extents
            .lines()
            .map(|line_id| {
                let line_span = line_id.span();

                let annotations: Vec<_> = error
                    .annotations()
                    .list()
                    .into_iter()
                    .flat_map(|annotations| annotations.iter())
                    .filter(|&annotation| Self::line_relevant(annotation.target(), line_span, source_size))
                    .collect();

                match OneOrMore::from_vec(annotations) {
                    Some(annotations) => SnippetSegment::Relevant(RelevantSegment { line_id, annotations }),
                    None => SnippetSegment::Irrelevant(IrrelevantSegment(line_id)),
                }
            })
            .collect();

        let mut line_iter = line_snippets.into_iter().peekable();

        let view_extents = viewport.extents();

        let mut marker = charset.indicator().marker();

        let annotation_count = error.annotations().list().map(|annotations| annotations.len().get()).unwrap_or(0);

        let mut marker_map: HashMap<Span, char> = HashMap::with_capacity(annotation_count);

        let mut style_map: HashMap<Span, Style> = HashMap::with_capacity(annotation_count);

        let (default_style, mut style_iter) = theme.arrow();

        while let Some::<SnippetSegment<'_, '_, E>>(line_segment) = line_iter.next() {
            match line_segment {
                SnippetSegment::Relevant(segment) => {
                    let id = segment.id();

                    let target_list = segment.annotations();

                    match id.content() {
                        Some(line_content) => {
                            let content_span = line_content.span();
                            let line = line_content.value();

                            let mut burst_iter = content_span
                                .range()
                                .step_by(view_extents.width() as _)
                                .into_iter()
                                .map(|start| Span::new(start, view_extents.width_raw().into()))
                                .map(|span| span.intersect(content_span))
                                .map(|span| span.map(|span| (span, line.window(span.normal()))))
                                .map(|opt| opt.map(|(span, view)| view.map(|view| (span, view))))
                                .map(Option::flatten)
                                .flatten();

                            let mut is_first = true;

                            while let Some((phase_span, phase_view)) = burst_iter.next() {
                                if is_first {
                                    is_first = false;

                                    ' '.times(line_col_padding.left())
                                        .sequence(id.number().stylable().styled(theme.lines()).only_when(is_colored))
                                        .sequence(' '.times(line_col_padding.right()))
                                        .print(sink)?;
                                } else {
                                    ' '.times(line_col_width).print(sink)?;
                                }

                                charset.delimit().left().sequence(' ').print(sink)?;

                                phase_view.visualize(sink)?;

                                '\n'.print(sink)?;

                                let mut target_list = {
                                    let mut target_iter = target_list.iter().filter_map(|&annotation| {
                                        Self::phase_target(annotation.target(), phase_span, content_span).map(|target| (target, annotation))
                                    });

                                    let (capacity, _) = target_iter.size_hint();

                                    let mut target_list = Vec::with_capacity(capacity);

                                    while let Some(target_value) = target_iter.next() {
                                        target_list.push(target_value);
                                    }

                                    target_list
                                };

                                target_list.sort_by_key(|(target, ..)| Self::display_end(*target));

                                let target_backup: Vec<(Span, &<E::Annotations as Annotations>::Annotation)> = target_list
                                    .iter()
                                    .map(|&(_, annotation)| (annotation.target(), annotation))
                                    .collect();

                                // Intermediate storage for printing operations.
                                let mut target_storage = Vec::new();

                                while let Some(_) = target_list.first() {
                                    (' '.times(line_col_width))
                                        .sequence(charset.delimit().left())
                                        .sequence(' ')
                                        .print(sink)?;

                                    let mut rightmost_index = phase_span.start();

                                    let mut target_ignore = Vec::new();

                                    for pair @ (target, _) in target_list.into_iter() {
                                        if Self::display_start(target) >= rightmost_index {
                                            target_storage.push(pair);

                                            rightmost_index = Self::display_end(target);
                                        } else {
                                            target_ignore.push(pair);
                                        }
                                    }

                                    target_list = target_ignore;

                                    let mut rightmost_index = phase_span.start();

                                    for (target, annotation) in target_storage.drain(..) {
                                        let annotation_target = annotation.target();

                                        let marker_char = marker_map
                                            .entry(annotation_target)
                                            .or_insert_with(|| marker.next().unwrap_or(marker.default()));

                                        let style = style_map
                                            .entry(annotation_target)
                                            .or_insert_with(|| style_iter.next().unwrap_or(default_style));

                                        ' '.times(Self::display_width(
                                            line,
                                            content_span,
                                            rightmost_index,
                                            Self::display_start(target),
                                        ))
                                        .sequence(
                                            marker_char
                                                .times(Self::marker_width(line, content_span, target))
                                                .stylable()
                                                .styled(*style)
                                                .only_when(is_colored),
                                        )
                                        .print(sink)?;

                                        rightmost_index = Self::display_end(target);
                                    }

                                    '\n'.print(sink)?
                                }

                                for (target, v) in target_backup {
                                    let is_end = line_iter
                                        .peek()
                                        .map(|snippet| !Self::line_relevant(target, snippet.id().span(), source_size))
                                        .unwrap_or(true);

                                    if is_end {
                                        let marker_char = marker_map
                                            .entry(target)
                                            .or_insert_with(|| marker.next().unwrap_or(marker.default()));

                                        let style = style_map
                                            .entry(target)
                                            .or_insert_with(|| style_iter.next().unwrap_or(default_style));

                                        ' '.times(line_col_width)
                                            .sequence(charset.delimit().left())
                                            .sequence(' ')
                                            .sequence(' '.times(2))
                                            .sequence(
                                                '['.stylable()
                                                    .bold()
                                                    .only_when(is_colored)
                                                    .sequence(marker_char.stylable().styled(*style).only_when(is_colored))
                                                    .sequence(']'.stylable().bold().only_when(is_colored)),
                                            )
                                            .sequence(':')
                                            .sequence(' ')
                                            .sequence(v.message().content())
                                            .sequence('\n')
                                            .sequence(' '.times(line_col_width).sequence(charset.delimit().left()).sequence(' '))
                                            .sequence('\n')
                                            .print(sink)?;
                                    }
                                }
                            }
                        }
                        None => {}
                    }
                }
                SnippetSegment::Irrelevant(_) => (),
            }
        }

        charset
            .delimit()
            .bottom()
            .times(line_col_width)
            .sequence(charset.delimit().corner().bottom_right())
            .print(sink)?;

        Ok(())
    }
}
