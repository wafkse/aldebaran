//! Proven line terminators and their absolute source extents.
//!
//! [`LineBreak`] stores the source-specific terminator view together with the span
//! that produced a successful terminator transition. It can be mapped or coerced
//! while retaining the original source coordinates.

use core::{cmp, fmt, hash};

use aldebaran_print::prelude::Print;
use aldebaran_span::prelude::{Span, Spanned};

use crate::line::SourceLines;

/// A line break that represents a successful state transition.
pub struct LineBreak<'a, S>
where
    S: SourceLines<'a> + ?Sized,
{
    /// The line break itself. Does not contain the rest of the line.
    line_break: S::Line,

    /// The span of the line break.
    span: Span,
}

impl<'a, S> Print for LineBreak<'a, S>
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
        let &Self { ref line_break, .. } = self;

        line_break.print_with_ctx(writer, context)
    }
}

impl<'a, S> hash::Hash for LineBreak<'a, S>
where
    S: SourceLines<'a> + ?Sized,
    S::Line: hash::Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let &Self { ref line_break, ref span } = self;

        line_break.hash(state);
        span.hash(state);
    }
}

impl<'a, S> Ord for LineBreak<'a, S>
where
    S: SourceLines<'a> + ?Sized,
    S::Line: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let &Self { ref line_break, ref span } = self;

        let &Self {
            line_break: ref other_line_break,
            span: ref other_span,
        } = other;

        match line_break.cmp(other_line_break) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        span.cmp(other_span)
    }
}

impl<'a, S> PartialOrd for LineBreak<'a, S>
where
    S: SourceLines<'a> + ?Sized,
    S::Line: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let &Self { ref line_break, ref span } = self;

        let &Self {
            line_break: ref other_line_break,
            span: ref other_span,
        } = other;

        match line_break.partial_cmp(other_line_break) {
            Some(cmp::Ordering::Equal) => span.partial_cmp(other_span),
            Some(ord) => Some(ord),
            None => None,
        }
    }
}

impl<'a, S> Eq for LineBreak<'a, S>
where
    S: SourceLines<'a> + ?Sized,
    S::Line: Eq,
{
}

impl<'a, S> PartialEq for LineBreak<'a, S>
where
    S: SourceLines<'a> + ?Sized,
    S::Line: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let &Self { ref line_break, ref span } = self;

        let &Self {
            line_break: ref other_line_break,
            span: ref other_span,
        } = other;

        line_break == other_line_break && span == other_span
    }
}

impl<'a, S> Copy for LineBreak<'a, S>
where
    S: SourceLines<'a> + ?Sized,
    S::Line: Copy,
{
}

impl<'a, S> Clone for LineBreak<'a, S>
where
    S: SourceLines<'a> + ?Sized,
    S::Line: Clone,
{
    fn clone(&self) -> Self {
        let &Self { ref line_break, span } = self;

        let line_break = line_break.clone();

        Self { line_break, span }
    }
}

impl<'a, S> fmt::Debug for LineBreak<'a, S>
where
    S: SourceLines<'a> + ?Sized,
    S::Line: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let &Self { ref line_break, ref span } = self;

        f.debug_struct("LineBreak")
            .field("line_break", line_break)
            .field("span", span)
            .finish()
    }
}

impl<'a, S> Spanned for LineBreak<'a, S>
where
    S: SourceLines<'a> + ?Sized,
{
    #[inline]

    fn span(&self) -> Span {
        let &Self { span, .. } = self;

        span
    }
}

impl<'a, S> LineBreak<'a, S>
where
    S: SourceLines<'a> + ?Sized,
{
    /// Construct a new [`LineBreak`] from a specific line terminator.
    #[inline]
    pub fn tuple((line, span): (S::Line, Span)) -> Self {
        Self { line_break: line, span }
    }
}

impl<'a, S> LineBreak<'a, S>
where
    S: SourceLines<'a> + ?Sized,
{
    /// Retrieve the terminator responsible for this line break.
    #[inline]
    pub const fn line_break(&self) -> &S::Line {
        let &Self { line_break: ref line, .. } = self;

        line
    }

    /// Retrieve the [`Span`] that arches over this line break.
    #[inline]
    pub const fn span(&self) -> Span {
        let &Self { span, .. } = self;

        span
    }

    /// Destruct this line break into its raw parts: the line break and the
    /// span.
    #[inline]
    pub fn into_raw_parts(self) -> (S::Line, Span) {
        let Self { line_break, span } = self;

        (line_break, span)
    }

    /// Coerce this line break's source type to another which has the same kind
    /// of line.
    #[inline]
    pub fn coerce<O>(self) -> LineBreak<'a, O>
    where
        O: SourceLines<'a, Line = S::Line> + ?Sized,
    {
        let Self { line_break, span } = self;

        LineBreak::tuple((line_break, span))
    }

    /// Map the contents of this line to another type.
    #[inline]
    pub fn map<F, O>(self, map: F) -> LineBreak<'a, O>
    where
        F: FnOnce(S::Line) -> O::Line,
        O: SourceLines<'a> + ?Sized,
    {
        let Self { line_break, span } = self;

        let line_break = map(line_break);

        LineBreak::tuple((line_break, span))
    }
}
