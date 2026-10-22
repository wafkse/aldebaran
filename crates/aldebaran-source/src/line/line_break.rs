//! Recognized line breaks with their original source positions.
//!
//! [`LineBreak`] keeps the terminator separate from [`LineContent`](crate::line::LineContent),
//! allowing a line's combined extent to include the break while rendering only
//! its content.

use core::{cmp, fmt, hash};

use aldebaran_baseline::prelude::{Span, Spanned};
use aldebaran_print::prelude::Print;

use crate::line::SourceLines;

/// A recognized terminator and its nonempty absolute source extent.
///
/// The view preserves the break's components, including multi-component breaks
/// such as CRLF. Printing emits those components without the preceding content.
// NOTE(invariant): Span = OriginalSourceRange(LineBreak)
pub struct LineBreak<'a, S>
where
    S: SourceLines<'a> + ?Sized,
{
    /// The source-specific view of the terminator.
    line_break: S::Line,

    /// The source-coordinate span associated with the terminator view.
    span: Span,
}

impl<'a, S> Print for LineBreak<'a, S>
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
        let &Self { ref line_break, .. } = self;

        line_break.print_with_context(writer, context)
    }
}

impl<'a, S> hash::Hash for LineBreak<'a, S>
where
    S: SourceLines<'a> + ?Sized,
    S::Line: hash::Hash,
{
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
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
    /// Pair a terminator view with its source-coordinate span.
    ///
    /// The caller must ensure the view corresponds to that span.
    #[inline]
    pub fn tuple((line, span): (S::Line, Span)) -> Self {
        Self { line_break: line, span }
    }
}

impl<'a, S> LineBreak<'a, S>
where
    S: SourceLines<'a> + ?Sized,
{
    /// Break components excluded from the line's renderable content.
    #[inline]
    pub const fn line_break(&self) -> &S::Line {
        let &Self { line_break: ref line, .. } = self;

        line
    }

    /// Absolute break extent with an exclusive end, measured in source units.
    #[inline]
    pub const fn span(&self) -> Span {
        let &Self { span, .. } = self;

        span
    }

    /// Separate the break representation from its absolute source extent.
    #[inline]
    pub fn partitioned(self) -> (S::Line, Span) {
        let Self { line_break, span } = self;

        (line_break, span)
    }

    /// Reassociate the break with a source type sharing the same line representation.
    /// Its view and absolute coordinates remain unchanged.
    #[inline]
    pub fn coerce<O>(self) -> LineBreak<'a, O>
    where
        O: SourceLines<'a, Line = S::Line> + ?Sized,
    {
        let Self { line_break, span } = self;

        LineBreak::tuple((line_break, span))
    }

    /// Adapt the break representation without rebasing its coordinates.
    /// The mapped view must still describe the same source region.
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
