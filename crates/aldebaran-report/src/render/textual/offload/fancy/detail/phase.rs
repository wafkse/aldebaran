//! Output rows within a source excerpt.
//!
//! A long source line can occupy several [`Phase`] values, which share the
//! rendering context and original source coordinates.

use core::{
    cmp::Ordering,
    fmt, iter,
    iter::FusedIterator,
    num::{NonZero, NonZeroU32},
};

use alloc::{string::String, vec::Vec};

use aldebaran_ansi::{brush::Paintable, predicate::Should, prelude::Style};
use aldebaran_baseline::prelude::Span;
use aldebaran_print::{
    prelude::{Combine, Print},
    visualize::Visualize,
};
use aldebaran_source::prelude::{LineContent, LineSegmented, SourceLines};
use unicode_width::UnicodeWidthStr;

use crate::annotated::{Annotated, title::Title};
use crate::render::textual::{
    offload::fancy::{Colored, FancySettings, Theme},
    viewport::Viewport,
};

use super::{Context, Gutter};

/// A row of source context and the annotations displayed beneath it.
///
/// Source coordinates are retained. Text and markers, however, align in terminal
/// cells after the gutter.
///
/// # Remarks
///
/// A phase can also represent an EOF insertion without stored source content.
// NOTE(invariant): (Row = Content => Columns = SourceBoundaryColumns(Span, Text))
// And (Number In Some(Value) => Digits(Value) <= Gutter.DigitWidth)
pub(in crate::render::textual::offload::fancy::detail) struct Phase<'gutter> {
    /// Content shown at this source location.
    row: Row,

    /// Source line number, absent on continuation rows.
    number: Option<NonZero<usize>>,

    /// Alignment with the other rows of the excerpt.
    gutter: &'gutter Gutter,
}

/// Content shown by a source row, including an insertion with no stored text.
enum Row {
    /// A wrapped portion of a physical source line.
    Content {
        /// Absolute source extent displayed on this row.
        span: Span,

        /// Exclusive source endpoint of the line's content, before its break.
        content_end: usize,

        /// Text as it appears after the gutter.
        text: String,

        /// Correspondence between source boundaries and columns after the gutter.
        columns: Vec<(usize, u32)>,
    },
    /// The insertion position after a final line break.
    Terminal {
        /// Source coordinate of the insertion.
        boundary: usize,
    },
}

impl<'gutter> Phase<'gutter> {
    /// Create source context for an insertion after a final line break.
    #[inline]
    pub(in crate::render::textual::offload::fancy::detail) const fn terminal(
        boundary: usize,
        number: NonZero<usize>,
        gutter: &'gutter Gutter,
    ) -> Self {
        let row = Row::Terminal { boundary };
        let number = Some(number);

        Self { row, number, gutter }
    }

    /// Write the source row, numbering only the first row of a wrapped line.
    #[inline]
    pub(in crate::render::textual::offload::fancy::detail) fn line<W, T, I>(&self, context: &mut Context<'_, '_, W, T, I>) -> fmt::Result
    where
        W: fmt::Write,
        T: Theme,
    {
        let Self { row, number, gutter } = self;

        gutter.line(*number, context)?;

        match row {
            Row::Content { text, .. } => context.sink.write_str(text)?,
            Row::Terminal { .. } => {}
        }

        '\n'.print(context.sink)
    }

    /// Mark visible annotation ranges on separate rows when they overlap,
    /// retaining caller order for equal endpoints.
    pub(in crate::render::textual::offload::fancy::detail) fn marker<'a, A, W, T, I>(
        &self,
        annotations: impl Iterator<Item = &'a A>,
        context: &mut Context<'_, '_, W, T, I>,
    ) -> fmt::Result
    where
        A: Annotated + 'a,
        W: fmt::Write,
        T: Theme,
        I: Iterator<Item = Style>,
    {
        let Self { row, gutter, .. } = self;

        let mut remaining: Vec<_> = annotations
            .filter_map(|annotation| self.project(annotation.target()).map(|target| (target, annotation)))
            .collect();

        remaining.sort_by_key(|(target, _)| target.end());

        let lanes = Lanes { remaining };

        for lane in lanes {
            gutter.line(None, context)?;

            let mut rightmost = match row {
                Row::Content { span, .. } => span.start(),
                Row::Terminal { boundary } => *boundary,
            };

            for (target, annotation) in lane {
                let (marker, style) = context.marker(annotation.target());

                let padding = self.width(rightmost, target.start());
                let width = self.width(target.start(), target.end()).max(1);
                let settings = context.settings;
                let colored = Should::tuple((settings.colored_ref(), Colored::colored));

                ' '.times(padding)
                    .sequence(marker.times(width).stylable().styled(style).only_when(&colored))
                    .print(context.sink)?;

                rightmost = target.end();
            }

            '\n'.print(context.sink)?;
        }

        Ok(())
    }

    /// Display a message at its annotation's final visible position in this line.
    ///
    /// The caller must defer it if the annotation continues onto a later source line.
    pub(in crate::render::textual::offload::fancy::detail) fn message<A, W, T, I>(
        &self,
        annotation: &A,
        context: &mut Context<'_, '_, W, T, I>,
    ) -> fmt::Result
    where
        A: Annotated,
        <A::Title as Title>::Context: Default,
        W: fmt::Write,
        T: Theme,
        I: Iterator<Item = Style>,
    {
        let Self { row, gutter, .. } = self;

        let target = annotation.target();
        let is_final = match row {
            Row::Content { span, content_end, .. } => target.end() <= span.end() || span.end() == *content_end,
            Row::Terminal { .. } => true,
        };

        match self.project(target).filter(|_| is_final) {
            Some(_) => {
                let (marker, style) = context.marker(target);

                let settings = context.settings;
                let colored = Should::tuple((settings.colored_ref(), Colored::colored));

                gutter.line(None, context)?;

                match row {
                    Row::Content { .. } => {
                        "  ".sequence('['.stylable().bold().only_when(&colored))
                            .sequence(marker.stylable().styled(style).only_when(&colored))
                            .sequence(']'.stylable().bold().only_when(&colored))
                            .sequence(": ")
                            .sequence(annotation.message().content())
                            .sequence('\n')
                            .print(context.sink)?;

                        gutter.line(None, context)?;

                        '\n'.print(context.sink)
                    }
                    Row::Terminal { .. } => "  ["
                        .sequence(marker.stylable().styled(style).only_when(&colored))
                        .sequence("]: ")
                        .sequence(annotation.message().content())
                        .sequence('\n')
                        .print(context.sink),
                }
            }
            None => Ok(()),
        }
    }

    /// Clip a target to this row, including an insertion at the end of its source line.
    #[inline]
    fn project(&self, target: Span) -> Option<Span> {
        let Self { row, .. } = self;

        match row {
            Row::Content { span, content_end, .. } => match target.intersect(*span) {
                Some(projected) => Some(projected),
                None => (target == Span::unit(*content_end) && span.end() == *content_end).then_some(target),
            },
            Row::Terminal { boundary } => (target == Span::unit(*boundary)).then_some(target),
        }
    }

    /// Display width between source boundaries, or zero if either has no column.
    #[inline]
    fn width(&self, start: usize, end: usize) -> u32 {
        let Self { row, .. } = self;

        match row {
            Row::Content { columns, .. } => {
                let column = |index| {
                    columns
                        .binary_search_by_key(&index, |(source, _)| *source)
                        .ok()
                        .and_then(|index| columns.get(index))
                        .map(|(_, column)| *column)
                };

                match (column(start), column(end)) {
                    (Some(start), Some(end)) => end.saturating_sub(start),
                    _ => 0,
                }
            }
            Row::Terminal { .. } => 0,
        }
    }
}

/// Marker rows whose annotation ranges do not overlap.
// NOTE(invariant): StableSortedByEnd(Remaining)
struct Lanes<'a, A> {
    /// Targets awaiting a row, ordered by their endpoint.
    remaining: Vec<(Span, &'a A)>,
}

impl<'a, A> Iterator for Lanes<'a, A> {
    type Item = Vec<(Span, &'a A)>;

    fn next(&mut self) -> Option<Self::Item> {
        let &mut Self { ref mut remaining } = self;

        let mut lane = Vec::new();
        let mut overlapping = Vec::new();
        let mut rightmost = remaining.first().map(|(span, _)| span.start())?;

        for target @ (span, _) in remaining.drain(..) {
            if span.start() >= rightmost {
                rightmost = span.end();
                lane.push(target);
            } else {
                overlapping.push(target);
            }
        }

        *remaining = overlapping;

        Some(lane)
    }
}

impl<A> FusedIterator for Lanes<'_, A> {}

/// Wrapped rows for the content of a physical source line.
///
/// # Remarks
///
/// A row can exceed the viewport width when its first source component is wider,
/// so wrapping preserves that component rather than splitting it.
// NOTE(invariant): Start In ValidSourceBoundaries(Content) Union {Content.Span.End}
pub(in crate::render::textual::offload::fancy::detail) struct Phases<'line, 'source, 'settings, S, T: Theme>
where
    S: SourceLines<'source> + ?Sized,
{
    /// Line content in the original source coordinates.
    content: &'line LineContent<'source, S>,

    /// Source coordinate at which the next row begins.
    start: usize,

    /// Source line number shown on the first wrapped row.
    number: NonZero<usize>,

    /// Alignment with the rest of the excerpt.
    gutter: &'line Gutter,

    /// Presentation policy for the line's text.
    settings: &'settings FancySettings<T>,

    /// Available row width in terminal cells.
    viewport: &'settings Viewport,
}

impl<'line, 'source, 'settings, S, T: Theme> Phases<'line, 'source, 'settings, S, T>
where
    S: SourceLines<'source> + ?Sized,
{
    /// Wrap the line using the render's settings and viewport.
    #[inline]
    pub(in crate::render::textual::offload::fancy::detail) const fn new(
        content: &'line LineContent<'source, S>,
        number: NonZero<usize>,
        gutter: &'line Gutter,
        settings: &'settings FancySettings<T>,
        viewport: &'settings Viewport,
    ) -> Self {
        let start = content.span().start();

        Self {
            content,
            start,
            number,
            gutter,
            settings,
            viewport,
        }
    }
}

impl<'line, 'source, S, T: Theme> Iterator for Phases<'line, 'source, '_, S, T>
where
    S: SourceLines<'source> + ?Sized,
    <<S as SourceLines<'source>>::Line as LineSegmented<'source, S>>::View: Visualize,
{
    type Item = Result<Phase<'line>, fmt::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let &mut Self {
            content,
            ref mut start,
            number,
            gutter,
            settings,
            viewport,
        } = self;

        let content_span = content.span();

        match (*start).cmp(&content_span.end()) {
            Ordering::Less => {
                let text = String::new();
                let mut visible = VisibleText { text, settings };
                let mut columns = Vec::new();
                let mut end = *start;
                let mut candidate = *start;
                let mut error = None;

                columns.push((end, visible.width()));

                while candidate < content_span.end() {
                    candidate += 1;

                    let Some(length) = NonZero::new(candidate - end) else {
                        break;
                    };

                    let relative = Span::new(end - content_span.start(), length);

                    match content.value().window(relative) {
                        Some(view) => {
                            let checkpoint = visible.text.len();

                            match view.visualize(&mut visible) {
                                Ok(()) => {
                                    if visible.width() > u32::from(viewport.extents().width()) && end > *start {
                                        visible.text.truncate(checkpoint);
                                        break;
                                    }

                                    end = candidate;
                                    columns.push((end, visible.width()));
                                }
                                Err(failure) => {
                                    error = Some(failure);
                                    break;
                                }
                            }
                        }
                        None => continue,
                    }
                }
                match error {
                    Some(error) => {
                        *start = content_span.end();

                        Some(Err(error))
                    }
                    None => {
                        if let Some(length) = NonZero::new(end - *start) {
                            let span = Span::new(*start, length);
                            let content_end = content_span.end();
                            let text = visible.text;
                            let row = Row::Content {
                                span,
                                content_end,
                                text,
                                columns,
                            };
                            let number = (*start == content_span.start()).then_some(number);
                            let phase = Phase { row, number, gutter };

                            *start = end;

                            Some(Ok(phase))
                        } else {
                            *start = content_span.end();

                            None
                        }
                    }
                }
            }
            Ordering::Equal | Ordering::Greater => None,
        }
    }
}

impl<'source, S, T: Theme> FusedIterator for Phases<'_, 'source, '_, S, T>
where
    S: SourceLines<'source> + ?Sized,
    <<S as SourceLines<'source>>::Line as LineSegmented<'source, S>>::View: Visualize,
{
}

/// Source text with tabs expanded and terminal controls made visible.
// NOTE(invariant): None.
struct VisibleText<'settings, T: Theme> {
    /// Text displayed after the gutter.
    text: String,

    /// Tab-stop policy for this text.
    settings: &'settings FancySettings<T>,
}

impl<T: Theme> VisibleText<'_, T> {
    /// Terminal cells occupied by the text, saturating at `u32::MAX`.
    #[inline]
    fn width(&self) -> u32 {
        let Self { text, .. } = self;

        u32::try_from(text.width()).unwrap_or(u32::MAX)
    }
}

impl<T: Theme> fmt::Write for VisibleText<'_, T> {
    #[inline]
    fn write_str(&mut self, source: &str) -> fmt::Result {
        for character in source.chars() {
            self.write_char(character)?;
        }

        Ok(())
    }

    #[inline]
    fn write_char(&mut self, character: char) -> fmt::Result {
        let &mut Self { ref mut text, settings } = self;

        match character {
            '\t' => {
                let width = NonZeroU32::from(settings.tab_width()).get();
                let padding = width - u32::try_from(text.width()).unwrap_or(u32::MAX) % width;

                text.extend(iter::repeat_n(' ', padding as usize));
            }
            character if character.is_control() => text.push(char::REPLACEMENT_CHARACTER),
            character => text.push(character),
        }

        Ok(())
    }
}
