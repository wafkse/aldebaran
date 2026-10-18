//! Source line contents excluding their terminating line break.
//!
//! [`LineContent`] pairs the source-specific line view with its absolute span.
//! Mapping can change the source line representation while preserving those
//! coordinates, so renderers can adapt content while keeping annotation positions aligned.

use core::{cmp, fmt, hash};

use aldebaran_print::prelude::Print;

use aldebaran_baseline::prelude::{Span, Spanned};

use crate::line::SourceLines;

/// Nonempty line content with its position in the original source.
///
/// Printing uses the content view alone. The span remains available to align
/// annotations with that output and excludes the terminating line break.
// NOTE(invariant): The caller-supplied `span` must describe `line_content` in the original source coordinates.
pub struct LineContent<'a, S>
where
    S: SourceLines<'a> + ?Sized,
{
    /// The content of the line, excluding the line break.
    line_content: S::Line,

    /// The source-coordinate span associated with the content view.
    span: Span,
}

impl<'a, S> Print for LineContent<'a, S>
where
    S: SourceLines<'a> + ?Sized,
    S::Line: Print,
{
    type Context = <S::Line as Print>::Context;

    #[inline]
    fn print_with_context<W>(&self, writer: &mut W, context: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        let &Self { ref line_content, .. } = self;

        line_content.print_with_context(writer, context)
    }
}

impl<'a, S> LineContent<'a, S>
where
    S: SourceLines<'a> + ?Sized,
{
    /// Pair content with its source-coordinate span.
    ///
    /// The caller must ensure the span belongs to that view.
    #[inline]
    pub const fn from_raw_parts(line_content: S::Line, span: Span) -> Self {
        Self { line_content, span }
    }
}

impl<'a, S> LineContent<'a, S>
where
    S: SourceLines<'a> + ?Sized,
{
    /// Content available for rendering and line-local windows, excluding the break.
    #[inline]
    pub const fn value(&self) -> &S::Line {
        let &Self { ref line_content, .. } = self;

        line_content
    }

    /// Absolute content extent with an exclusive end, measured in source units.
    #[inline]
    pub const fn span(&self) -> Span {
        let &Self { span, .. } = self;

        span
    }

    /// Adapt the content representation without rebasing its coordinates.
    /// The mapped view must still describe the same source region.
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
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
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
        let Self { line_content, span } = self;

        f.debug_struct("LineContent")
            .field("line_content", line_content)
            .field("span", span)
            .finish()
    }
}
