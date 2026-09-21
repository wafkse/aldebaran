//! Source line contents excluding their terminating line break.
//!
//! [`LineContent`] pairs the source-specific line view with its absolute span.
//! Mapping can change the source line representation while preserving those
//! coordinates, which lets renderers adapt content without losing source identity.

use core::{cmp, fmt, hash};

use aldebaran_print::prelude::Print;

use aldebaran_span::prelude::{Span, Spanned};

use crate::line::SourceLines;

/// Content of a line in a source.
pub struct LineContent<'a, S>
where
    S: SourceLines<'a> + ?Sized,
{
    /// The content of the line, excluding the line break.
    line_content: S::Line,
    /// The span that arches over the line's content.
    span: Span,
}

impl<'a, S> Print for LineContent<'a, S>
where
    S: SourceLines<'a> + ?Sized,
    S::Line: Print,
{
    type Context = <S::Line as Print>::Context;

    #[inline]
    fn print_with_ctx<W>(&self, writer: &mut W, context: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        let &Self { ref line_content, .. } = self;

        line_content.print_with_ctx(writer, context)
    }
}

impl<'a, S> LineContent<'a, S>
where
    S: SourceLines<'a> + ?Sized,
{
    /// Create a new [`LineContent`] from the raw parts: the line content and
    /// its englobing span.
    #[inline]
    pub const fn from_raw_parts(line_content: S::Line, span: Span) -> Self {
        Self { line_content, span }
    }
}

impl<'a, S> LineContent<'a, S>
where
    S: SourceLines<'a> + ?Sized,
{
    /// Retrieve the content of the line.
    #[inline]
    pub const fn value(&self) -> &S::Line {
        let &Self { ref line_content, .. } = self;

        line_content
    }

    /// Retrieve the span that arches over the line's content.
    #[inline]
    pub const fn span(&self) -> Span {
        let &Self { span, .. } = self;

        span
    }

    /// Map the contents of this line to another type.
    #[inline]
    pub fn map<F, O>(self, map: F) -> LineContent<'a, O>
    where
        F: FnOnce(S::Line) -> O::Line,
        O: SourceLines<'a> + ?Sized,
    {
        let Self { line_content, span } = self;

        let line_content = map(line_content);

        LineContent::from_raw_parts(line_content, span)
    }
}

impl<'a, S> Spanned for LineContent<'a, S>
where
    S: SourceLines<'a> + ?Sized,
{
    #[inline]
    fn span(&self) -> Span {
        let &Self { span, .. } = self;

        span
    }
}

impl<'a, S> hash::Hash for LineContent<'a, S>
where
    S: SourceLines<'a> + ?Sized,
    S::Line: hash::Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let &Self {
            ref line_content,
            ref span,
        } = self;

        line_content.hash(state);
        span.hash(state);
    }
}

impl<'a, S> Ord for LineContent<'a, S>
where
    S: SourceLines<'a> + ?Sized,
    S::Line: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let &Self {
            ref line_content,
            ref span,
        } = self;

        let &Self {
            line_content: ref other_line_content,
            span: ref other_span,
        } = other;

        match line_content.cmp(other_line_content) {
            cmp::Ordering::Equal => span.cmp(other_span),
            target_value => target_value,
        }
    }
}

impl<'a, S> PartialOrd for LineContent<'a, S>
where
    S: SourceLines<'a> + ?Sized,
    S::Line: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let &Self {
            ref line_content,
            ref span,
        } = self;

        let &Self {
            line_content: ref other_line_content,
            span: ref other_span,
        } = other;

        match line_content.partial_cmp(other_line_content) {
            Some(cmp::Ordering::Equal) => span.partial_cmp(other_span),
            target_value => target_value,
        }
    }
}

impl<'a, S> Eq for LineContent<'a, S>
where
    S: SourceLines<'a> + ?Sized,
    S::Line: Eq,
{
}

impl<'a, S> PartialEq for LineContent<'a, S>
where
    S: SourceLines<'a> + ?Sized,
    S::Line: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let &Self {
            ref line_content,
            ref span,
        } = self;

        let &Self {
            line_content: ref other_line_content,
            span: ref other_span,
        } = other;

        line_content == other_line_content && span == other_span
    }
}

impl<'a, S> Copy for LineContent<'a, S>
where
    S: SourceLines<'a> + ?Sized,
    S::Line: Copy,
{
}

impl<'a, S> Clone for LineContent<'a, S>
where
    S: SourceLines<'a> + ?Sized,
    S::Line: Clone,
{
    fn clone(&self) -> Self {
        let &Self { ref line_content, span } = self;

        let line_content = line_content.clone();

        Self { line_content, span }
    }
}

impl<'a, S> fmt::Debug for LineContent<'a, S>
where
    S: SourceLines<'a> + ?Sized,
    S::Line: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LineContent")
            .field("line_content", &self.line_content)
            .field("span", &self.span)
            .finish()
    }
}
