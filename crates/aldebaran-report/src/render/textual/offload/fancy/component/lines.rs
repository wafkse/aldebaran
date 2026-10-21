//! Annotated source excerpts for the [`Fancy`](super::super::Fancy) renderer.
//!
//! [`ByLines`] uses selected source extents to locate annotations and presents
//! their messages alongside line numbers. Layout works in terminal cells while
//! annotation targets retain the source's coordinate units.

mod snippet;

pub use self::snippet::{IrrelevantSegment, RelevantSegment, SnippetSegment};

use core::{fmt, hash, iter, marker, num::NonZero};

use alloc::{string::String, vec::Vec};
use unicode_width::UnicodeWidthStr;

use aldebaran_baseline::prelude::Nonempty;
use aldebaran_print::prelude::{Combine, Print};
use hashbrown::HashMap;

use aldebaran_ansi::{brush::Paintable, predicate::Should, prelude::Style};

use aldebaran_source::prelude::{LineExtent, LineSegmented, Location, Source, SourceLines, SourceMetadata, Terminator};

use aldebaran_baseline::prelude::{Span, Spanned};

use aldebaran_print::visualize::Visualize;

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

/// Rendered source text with visible terminal controls.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
// NOTE(invariant): None.
struct DisplayText(String);

impl DisplayText {
    /// Return the rendered width in terminal cells.
    #[inline]
    fn width(&self) -> u32 {
        let Self(text) = self;

        u32::try_from(text.width()).unwrap_or(u32::MAX)
    }
}

impl fmt::Write for DisplayText {
    #[inline]
    fn write_str(&mut self, source: &str) -> fmt::Result {
        for character in source.chars() {
            self.write_char(character)?;
        }

        Ok(())
    }

    #[inline]
    fn write_char(&mut self, character: char) -> fmt::Result {
        let tab_padding = match character {
            '\t' => 4 - self.width() % 4,
            _ => 0,
        };

        let Self(text) = self;

        match character {
            '\t' => text.extend(iter::repeat_n(' ', tab_padding as usize)),
            character if character.is_control() => text.push(char::REPLACEMENT_CHARACTER),
            character => text.push(character),
        }

        Ok(())
    }
}

/// Source lines and annotation messages rendered under Fancy's layout policy.
///
/// The backend input supplies selected source extents and the shared Fancy
/// settings. Source spans retain their original coordinates while wrapping and
/// marker placement follow the terminal-cell rules of
/// [`Fancy`](crate::render::textual::offload::fancy::Fancy).
// NOTE(invariant): None.
pub struct ByLines<'a, E, Te, Th = DefaultTheme>
where
    E: SourceReport<'a>,
    Th: Theme,
    Te: Terminator<'a, E::Source>,
    E::Source: SourceLines<'a> + SourceMetadata<'a>,
    <E::Source as SourceMetadata<'a>>::Metadata: Location,
{
    /// Type information for the report, source, terminator, and theme.
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
        *self
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
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
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
    /// Whether a target selects the line or its EOF boundary.
    #[inline]
    fn line_relevant(target: Span, line_span: Span, source_size: usize) -> bool {
        if target.overlaps(line_span) {
            true
        } else if target == Span::unit(source_size) {
            line_span.end() == source_size
        } else {
            false
        }
    }

    /// Project a target onto a phase, preserving a final EOF insertion.
    #[inline]
    fn phase_target(target: Span, phase_span: Span, content_span: Span) -> Option<Span> {
        match target.intersect(phase_span) {
            Some(span) => Some(span),
            None => {
                let boundary = target == Span::unit(content_span.end());
                let final_phase = phase_span.end() == content_span.end();

                match (boundary, final_phase) {
                    (true, true) => Some(target),
                    _ => None,
                }
            }
        }
    }

    /// Return the rendered width from the phase start through `end`.
    #[inline]
    fn display_prefix(line: &<E::Source as SourceLines<'a>>::Line, content_span: Span, origin: usize, end: usize) -> u32
    where
        <<E::Source as SourceLines<'a>>::Line as LineSegmented<'a, E::Source>>::View: Visualize,
    {
        let end = end.min(content_span.end());

        match NonZero::new(end.saturating_sub(origin)) {
            Some(length) => {
                let relative = Span::new(origin - content_span.start(), length);
                let mut text = DisplayText::default();

                match line.window(relative) {
                    Some(view) => {
                        let _: fmt::Result = view.visualize(&mut text);

                        text.width()
                    }
                    None => 0,
                }
            }
            None => 0,
        }
    }

    /// Return the rendered width between `start` and `end` in this phase.
    #[inline]
    fn display_width(line: &<E::Source as SourceLines<'a>>::Line, content_span: Span, origin: usize, start: usize, end: usize) -> u32
    where
        <<E::Source as SourceLines<'a>>::Line as LineSegmented<'a, E::Source>>::View: Visualize,
    {
        Self::display_prefix(line, content_span, origin, end).saturating_sub(Self::display_prefix(line, content_span, origin, start))
    }

    /// Return a source window within the terminal-cell budget when possible.
    /// The first component remains intact if it exceeds the budget.
    fn phase(
        line: &<E::Source as SourceLines<'a>>::Line,
        content_span: Span,
        start: usize,
        budget: u32,
    ) -> Result<Option<(Span, String)>, fmt::Error>
    where
        <<E::Source as SourceLines<'a>>::Line as LineSegmented<'a, E::Source>>::View: Visualize,
    {
        let mut end = start;
        let mut text = DisplayText::default();
        let mut candidate = start;

        while candidate < content_span.end() {
            candidate += 1;

            let length = NonZero::new(candidate - end).expect("candidate advances beyond the previous endpoint");
            let relative = Span::new(end - content_span.start(), length);

            match line.window(relative) {
                Some(view) => {
                    let DisplayText(ref buffer) = text;

                    let previous_len = buffer.len();

                    view.visualize(&mut text)?;

                    if text.width() > budget && end > start {
                        let DisplayText(ref mut buffer) = text;

                        buffer.truncate(previous_len);
                        break;
                    }

                    end = candidate;
                }
                None => continue,
            }
        }

        let DisplayText(text) = text;

        let span = NonZero::new(end - start).map(|length| Span::new(start, length));

        Ok(span.map(|span| (span, text)))
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

    #[expect(
        clippy::missing_inline_in_public_items,
        reason = "Source selection, wrapping, and annotation rendering are substantial formatting work."
    )]
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

        let is_colored = &Should::tuple((settings.colored_ref(), Colored::colored));

        let theme = settings.theme();

        let layout = settings.layout().unwrap_or_else(|| LayoutSettings::viewport(&viewport));

        let charset = settings.charset();

        let metadata = extents.metadata();

        let line_range = metadata.lines();

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

                let mut annotations = annotations.into_iter();
                let annotations = annotations.next().map(|first| {
                    let rest = annotations.collect();

                    Nonempty::multiple(first, rest)
                });

                match annotations {
                    Some(annotations) => SnippetSegment::Relevant(RelevantSegment { line_id, annotations }),
                    None => SnippetSegment::Irrelevant(IrrelevantSegment(line_id)),
                }
            })
            .collect();

        let terminal_line = line_snippets.last().and_then(|snippet| {
            let id = snippet.id();
            let has_boundary = error
                .annotations()
                .list()
                .is_some_and(|annotations| annotations.iter().any(|annotation| annotation.target() == Span::unit(source_size)));

            (id.span().end() == source_size && id.content_break().is_some() && has_boundary).then(|| id.number().get() + 1)
        });
        let last_displayed = terminal_line.unwrap_or(line_range.end().get());
        let line_no_width = last_displayed.ilog10() + 1;

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

                            let mut phase_start = content_span.start();
                            let burst_iter =
                                iter::from_fn(
                                    || match Self::phase(line, content_span, phase_start, u32::from(view_extents.width())) {
                                        Ok(Some((span, text))) => {
                                            phase_start = span.end();

                                            Some(Ok((span, text)))
                                        }
                                        Ok(None) => None,
                                        Err(error) => {
                                            phase_start = content_span.end();

                                            Some(Err(error))
                                        }
                                    },
                                );

                            let mut is_first = true;

                            for phase in burst_iter {
                                let (phase_span, phase_view) = phase?;

                                if is_first {
                                    is_first = false;

                                    ' '.times(line_col_padding.left() + line_no_width - (id.number().get().ilog10() + 1))
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
                                    let target_iter = target_list.iter().filter_map(|&annotation| {
                                        Self::phase_target(annotation.target(), phase_span, content_span).map(|target| (target, annotation))
                                    });

                                    let (capacity, _) = target_iter.size_hint();

                                    let mut target_list = Vec::with_capacity(capacity);

                                    for target_value in target_iter {
                                        target_list.push(target_value);
                                    }

                                    target_list
                                };

                                target_list.sort_by_key(|(target, ..)| target.end());

                                let target_backup: Vec<(Span, &<E::Annotations as Annotations>::Annotation)> = target_list
                                    .iter()
                                    .map(|&(_, annotation)| (annotation.target(), annotation))
                                    .collect();

                                // Intermediate storage for printing operations.
                                let mut target_storage = Vec::new();

                                while !target_list.is_empty() {
                                    (' '.times(line_col_width))
                                        .sequence(charset.delimit().left())
                                        .sequence(' ')
                                        .print(sink)?;

                                    let mut rightmost_index = phase_span.start();

                                    let mut target_ignore = Vec::new();

                                    for pair @ (target, _) in target_list.into_iter() {
                                        if target.start() >= rightmost_index {
                                            target_storage.push(pair);

                                            rightmost_index = target.end();
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

                                        let padding =
                                            Self::display_width(line, content_span, phase_span.start(), rightmost_index, target.start());
                                        let width =
                                            Self::display_width(line, content_span, phase_span.start(), target.start(), target.end())
                                                .max(1);

                                        ' '.times(padding)
                                            .sequence(marker_char.times(width).stylable().styled(*style).only_when(is_colored))
                                            .print(sink)?;

                                        rightmost_index = target.end();
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

                    if id.span().end() == source_size
                        && let Some(number) = terminal_line
                    {
                        ' '.times(line_col_padding.left())
                            .sequence(number.stylable().styled(theme.lines()).only_when(is_colored))
                            .sequence(' '.times(line_col_padding.right()))
                            .sequence(charset.delimit().left())
                            .sequence(' ')
                            .sequence('\n')
                            .print(sink)?;

                        for annotation in target_list
                            .iter()
                            .filter(|annotation| annotation.target() == Span::unit(source_size))
                        {
                            let target = annotation.target();
                            let marker_char = marker_map
                                .entry(target)
                                .or_insert_with(|| marker.next().unwrap_or(marker.default()));
                            let style = style_map
                                .entry(target)
                                .or_insert_with(|| style_iter.next().unwrap_or(default_style));

                            ' '.times(line_col_width)
                                .sequence(charset.delimit().left())
                                .sequence(' ')
                                .sequence(marker_char.stylable().styled(*style).only_when(is_colored))
                                .sequence('\n')
                                .sequence(' '.times(line_col_width))
                                .sequence(charset.delimit().left())
                                .sequence("   [")
                                .sequence(marker_char.stylable().styled(*style).only_when(is_colored))
                                .sequence("]: ")
                                .sequence(annotation.message().content())
                                .sequence('\n')
                                .print(sink)?;
                        }
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
