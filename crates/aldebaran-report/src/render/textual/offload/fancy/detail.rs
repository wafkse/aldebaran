//! Render a diagnostic's heading and annotated source under the same presentation.
//!
//! [`Context`] keeps settings and annotation markers consistent throughout the
//! diagnostic.
//!
//! [`Excerpt`] supplies the source context. Each [`Phase`] presents a source row
//! and the annotations that belong beneath it.

mod phase;

use core::{fmt, num::NonZero, ops::RangeInclusive};

use alloc::vec::Vec;

use aldebaran_ansi::{brush::Paintable, predicate::Should, prelude::Style};
use aldebaran_baseline::prelude::{Span, Spanned};
use aldebaran_print::{
    prelude::{Combine, Print},
    visualize::Visualize,
};
use aldebaran_source::prelude::{LineExtent, LineId, LineSegmented, Location, Source, SourceLines, SourceMetadata, Terminator};
use hashbrown::HashMap;

use crate::{
    annotated::{Annotated, Annotations},
    prelude::{ErrorKind, Title},
    render::textual::{OffloadContext, Present, viewport::Viewport},
    report::{Report, SourceReport, connotate::Connotate},
};

use self::phase::{Phase, Phases};
use super::{Colored, Fancy, FancySettings, LayoutSettings, Theme, charset::indicate::Marker, layout::padding::HorizontalPadding};

/// The annotation type supplied by a report.
type Annotation<E> = <<E as Report>::Annotations as Annotations>::Annotation;

/// State for rendering a diagnostic consistently across its output rows.
///
/// Operations use the same settings and marker assignments, so an annotation
/// remains recognizable when it wraps or spans source lines.
// NOTE(invariant): ForAll(Span In Assigned): Assigned[Span] = FirstAssignment(Span, ThisRender)
struct Context<'settings, 'output, W, T: Theme, I> {
    /// Destination for this diagnostic's output.
    sink: &'output mut W,

    /// Presentation policy used by the heading and source excerpt.
    settings: &'settings FancySettings<T>,

    /// Available display area, measured in terminal cells.
    viewport: &'settings Viewport,

    /// Glyphs available for annotations without an assigned marker.
    markers: Marker<'static>,

    /// Styles available for annotations without an assigned marker.
    styles: I,

    /// Style for annotations after the theme's palette is exhausted.
    fallback: Style,

    /// Marker identities keyed by original source span.
    assigned: HashMap<Span, (char, Style)>,
}

impl<W, T: Theme, I: Iterator<Item = Style>> Context<'_, '_, W, T, I> {
    /// A span keeps its marker across source rows and messages.
    fn marker(&mut self, target: Span) -> (char, Style) {
        let &mut Self {
            ref mut markers,
            ref mut styles,
            fallback,
            ref mut assigned,
            ..
        } = self;

        *assigned.entry(target).or_insert_with(|| {
            let marker = markers.next().unwrap_or(markers.default());
            let style = styles.next().unwrap_or(fallback);

            (marker, style)
        })
    }
}

/// The diagnostic heading, independent of its source excerpt.
// NOTE(invariant): None.
struct Header<'source, E>(&'source E);

impl<'source, E: SourceReport<'source>> Header<'source, E> {
    /// Format the report's classification and title with the heading theme.
    fn render<W, T, I>(&self, context: &mut Context<'_, '_, W, T, I>) -> fmt::Result
    where
        W: fmt::Write,
        T: Theme,
        <<E::Kind as ErrorKind>::Descriptor as Print>::Context: Default,
        <E::Title as Title>::Context: Default,
    {
        let Self(report) = self;

        let settings = context.settings;
        let colored = Should::tuple((settings.colored_ref(), Colored::colored));
        let kind = report.kind();
        let theme = settings.theme().metadata(&kind.connotates());
        let title = report.title();

        kind.descriptor()
            .stylable()
            .styled(theme.kind())
            .only_when(&colored)
            .sequence(':')
            .sequence(' ')
            .sequence(title.content().stylable().styled(theme.title()).only_when(&colored))
            .sequence('\n')
            .print(context.sink)
    }
}

/// A source line together with the annotations that belong to it.
// NOTE(invariant): ForAll(Annotation In Annotations): Overlaps(Annotation.Target, Id.Span)
// Or (Annotation.Target = Unit(SourceEnd) And Id.Span.End = SourceEnd)
struct Line<'source, E: SourceReport<'source>> {
    /// Line number and absolute source extent.
    id: LineId<'source, E::Source>,

    /// Annotations in caller order, using the line's source coordinates.
    annotations: Vec<&'source Annotation<E>>,
}

/// Source context for the report's annotated locations.
///
/// Line numbers refer to the original input.
///
/// # Remarks
///
/// An insertion after a final line break has its own display row, although it
/// has no stored source content.
// NOTE(invariant): (Lines, Numbers, Terminal) = Describe(SameSelection(Source))
struct Excerpt<'source, E: SourceReport<'source>> {
    /// Original input that gives the annotations their coordinates.
    source: &'source E::Source,

    /// Source lines in their original order.
    lines: Vec<Line<'source, E>>,

    /// Inclusive line numbers used to identify the excerpt.
    numbers: RangeInclusive<NonZero<usize>>,

    /// Line number shown for an insertion after a final line break.
    terminal: Option<NonZero<usize>>,

    /// Alignment for the line numbers and source frame.
    gutter: Gutter,
}

impl<'source, E: SourceReport<'source>> Excerpt<'source, E> {
    /// Select the report's annotations within the supplied line extent.
    fn new<W, T, I, Te>(
        report: &'source E,
        extents: &<E::Source as SourceLines<'source>>::Extents<Te>,
        context: &Context<'_, '_, W, T, I>,
    ) -> Self
    where
        T: Theme,
        Te: Terminator<'source, E::Source>,
    {
        let source = report.source();
        let source_end = source.size();
        let lines: Vec<Line<'source, E>> = extents
            .lines()
            .map(|id| {
                let annotations = report
                    .annotations()
                    .list()
                    .into_iter()
                    .flat_map(|annotations| annotations.iter())
                    .filter(|annotation| {
                        let target = annotation.target();

                        target.overlaps(id.span()) || (target == Span::unit(source_end) && id.span().end() == source_end)
                    })
                    .collect();

                Line { id, annotations }
            })
            .collect();
        let range = extents.metadata().lines();
        let numbers = range.start()..=range.end();
        let terminal = lines.last().and_then(|Line { id, annotations }| {
            let is_boundary = annotations.iter().any(|annotation| annotation.target() == Span::unit(source_end));

            match (id.content_break(), is_boundary) {
                (Some(_), true) => id.number().checked_add(1),
                _ => None,
            }
        });
        let digit_width = terminal.unwrap_or(*numbers.end()).get().ilog10() + 1;
        let settings = context.settings;
        let layout = settings.layout().unwrap_or_else(|| LayoutSettings::viewport(context.viewport));
        let padding = layout.line_column_padding();
        let gutter = Gutter { digit_width, padding };

        Self {
            source,
            lines,
            numbers,
            terminal,
            gutter,
        }
    }

    /// Display the excerpt with each message after its final visible marker.
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
            lines,
            numbers,
            terminal,
            gutter,
        } = self;

        let source_end = source.size();
        let metadata = source.metadata();
        let mut lines = lines.iter();

        gutter.open(metadata, numbers, context)?;

        while let Some(Line { id, annotations }) = lines.next() {
            if annotations.is_empty() {
                continue;
            }

            if let Some(content) = id.content() {
                let messages: Vec<_> = annotations
                    .iter()
                    .copied()
                    .filter(|annotation| {
                        let target = annotation.target();
                        let is_terminal = terminal.is_some() && target == Span::unit(source_end);
                        let is_continued = lines.clone().any(|Line { id, .. }| {
                            id.content().is_some_and(|content| {
                                let span = content.span();

                                target.overlaps(span) || target == Span::unit(span.end())
                            })
                        });

                        !is_terminal && !is_continued
                    })
                    .collect();

                let settings = context.settings;
                let viewport = context.viewport;
                let number = id.number();
                let phases = Phases::new(content, number, &gutter, settings, viewport);
                for phase in phases {
                    let phase = phase?;

                    phase.line(context)?;
                    phase.marker(annotations.iter().copied(), context)?;

                    for &annotation in &messages {
                        phase.message(annotation, context)?;
                    }
                }
            }

            if let (true, Some(number)) = (id.span().end() == source_end, terminal) {
                let phase = Phase::terminal(source_end, number, &gutter);

                phase.line(context)?;
                phase.marker(annotations.iter().copied(), context)?;

                for annotation in annotations
                    .iter()
                    .copied()
                    .filter(|annotation| annotation.target() == Span::unit(source_end))
                {
                    phase.message(annotation, context)?;
                }
            }
        }

        gutter.close(context)
    }
}

/// Alignment for the excerpt's line numbers and frame.
// NOTE(invariant): ForAll(Number In Excerpt.DisplayedNumbers): Digits(Number) <= DigitWidth
struct Gutter {
    /// Width of the largest line number, in terminal cells.
    digit_width: u32,

    /// Space on either side of the line-number column.
    padding: HorizontalPadding,
}

impl Gutter {
    /// Column width before the frame delimiter, in terminal cells.
    #[inline]
    const fn width(&self) -> u32 {
        let &Self { digit_width, padding } = self;

        digit_width + padding.absolute()
    }

    /// A missing number keeps continuation and annotation rows aligned.
    fn line<W, T, I>(&self, number: Option<NonZero<usize>>, context: &mut Context<'_, '_, W, T, I>) -> fmt::Result
    where
        W: fmt::Write,
        T: Theme,
    {
        let &Self { digit_width, padding } = self;

        let settings = context.settings;
        let colored = Should::tuple((settings.colored_ref(), Colored::colored));

        match number {
            Some(number) => {
                ' '.times(padding.left() + digit_width - (number.get().ilog10() + 1))
                    .sequence(number.stylable().styled(settings.theme().lines()).only_when(&colored))
                    .sequence(' '.times(padding.right()))
                    .print(context.sink)?;
            }
            None => ' '.times(self.width()).print(context.sink)?,
        }

        settings.charset().delimit().left().sequence(' ').print(context.sink)
    }

    /// Identify the excerpt by its source origin and original line numbers.
    fn open<W, T, I, M>(&self, metadata: &M, numbers: RangeInclusive<NonZero<usize>>, context: &mut Context<'_, '_, W, T, I>) -> fmt::Result
    where
        W: fmt::Write,
        T: Theme,
        M: Print + ?Sized,
        M::Context: Default,
    {
        let settings = context.settings;
        let charset = settings.charset();
        let colored = Should::tuple((settings.colored_ref(), Colored::colored));
        let style = settings.theme().lines();

        ' '.times(self.width())
            .sequence(charset.delimit().corner().top_left())
            .sequence(charset.delimit().top())
            .sequence(
                '['.sequence(metadata)
                    .sequence(':')
                    .sequence(numbers.start().stylable().styled(style).only_when(&colored))
                    .sequence(':')
                    .sequence(numbers.end().stylable().styled(style).only_when(&colored))
                    .sequence(']'),
            )
            .sequence('\n')
            .print(context.sink)
    }

    /// Finish the source frame at the gutter boundary.
    #[inline]
    fn close<W, T, I>(&self, context: &mut Context<'_, '_, W, T, I>) -> fmt::Result
    where
        W: fmt::Write,
        T: Theme,
    {
        let charset = context.settings.charset();

        charset
            .delimit()
            .bottom()
            .times(self.width())
            .sequence(charset.delimit().corner().bottom_right())
            .print(context.sink)
    }
}

/// Format a report using the caller's destination and presentation settings.
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
        input: ref settings,
        ref viewport,
    } = present;

    let (fallback, styles) = settings.theme().arrow();

    let markers = settings.charset().indicator().marker();
    let assigned = HashMap::new();
    let mut context = Context {
        sink,
        settings,
        viewport,
        markers,
        styles,
        fallback,
        assigned,
    };
    let header = Header(report);

    header.render(&mut context)?;

    match report.target().and_then(|target| report.source().segment(target)) {
        Some(extents) => Excerpt::new(report, &extents, &context).render(&mut context),
        None => Ok(()),
    }
}
