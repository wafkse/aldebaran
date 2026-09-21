//! Stable handles for individual source lines.
//!
//! [`LineId`] combines a one-based line number with content and optional break
//! extents. Its internal shape distinguishes content-only, content-with-break,
//! and break-only lines while presenting one common span and access interface.

use core::{fmt, hash, num::NonZero};

use aldebaran_span::prelude::{Span, Spanned};

use crate::line::{LineBreak, LineContent, SourceLines};

/// An identifier for a [`line`](SourceLines::Line) in a [`source`](crate::source::Source).
///
/// This struct encapsulates all the necessary information to uniquely identify
/// a line in a source, as well as providing a way to access the line itself.
///
/// In other words, this struct is a *handle* that provides instant access to:
///
/// - The line number.
/// - The line itself.
/// - The [`Span`] that englobes the line.
/// - The [`terminator`](crate::terminate::Terminator) of the line.
pub struct LineId<'a, S>
where
    S: SourceLines<'a> + ?Sized,
{
    line_inner: LineIdInner<'a, S>,
}

impl<'a, S> Clone for LineId<'a, S>
where
    S: SourceLines<'a> + ?Sized,
    S::Line: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let &Self { ref line_inner } = self;

        let line_inner = line_inner.clone();

        Self { line_inner }
    }
}

impl<'a, S> Copy for LineId<'a, S>
where
    S: SourceLines<'a> + ?Sized,
    S::Line: Copy,
{
}

impl<'a, S> hash::Hash for LineId<'a, S>
where
    S: SourceLines<'a> + ?Sized,
    S::Line: hash::Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let &Self { ref line_inner } = self;

        line_inner.hash(state);
    }
}

impl<'a, S> Spanned for LineId<'a, S>
where
    S: SourceLines<'a> + ?Sized,
{
    #[inline]
    fn span(&self) -> Span {
        let &Self { ref line_inner, .. } = self;

        match line_inner {
            LineIdInner::Standalone { line_content, .. } => line_content.span(),
            LineIdInner::WithBreak {
                line_content, line_break, ..
            } => line_content.span().superset(line_break.span()),
            LineIdInner::BreakOnly { line_break, .. } => line_break.span(),
        }
    }
}

impl<'a, S> Eq for LineId<'a, S>
where
    S: SourceLines<'a> + ?Sized,
    S::Line: Eq,
{
}

impl<'a, S> PartialEq for LineId<'a, S>
where
    S: SourceLines<'a> + ?Sized,
    S::Line: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let &Self { ref line_inner } = self;

        let &Self {
            line_inner: ref other_line_inner,
        } = other;

        line_inner == other_line_inner
    }
}

impl<'a, S> fmt::Debug for LineId<'a, S>
where
    S: SourceLines<'a> + ?Sized,
    S::Line: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let &Self { ref line_inner } = self;

        f.debug_struct("LineId").field("line_inner", line_inner).finish()
    }
}

impl<'a, S> LineId<'a, S>
where
    S: SourceLines<'a> + ?Sized,
{
    /// Construct a new [`LineId`] from its raw parts: a line number, the line
    /// content, and its line break.
    #[inline]
    pub fn from_raw_parts(line_number: NonZero<usize>, line_content: LineContent<'a, S>, line_break: Option<LineBreak<'a, S>>) -> Self {
        let line_inner = match line_break {
            Some(line_break) => LineIdInner::WithBreak {
                line_number,
                line_content,
                line_break,
            },
            None => LineIdInner::Standalone { line_number, line_content },
        };

        Self { line_inner }
    }

    /// Create a new [`LineId`] from a standalone line, i.e, a line without a
    /// line break.
    #[inline]
    pub const fn standalone(line_number: NonZero<usize>, line_content: LineContent<'a, S>) -> Self {
        let line_inner = LineIdInner::Standalone { line_number, line_content };

        Self { line_inner }
    }

    /// Create a full-fledged [`LineId`] from a line with content and a line
    /// break.
    #[inline]
    pub const fn full(line_number: NonZero<usize>, line_content: LineContent<'a, S>, line_break: LineBreak<'a, S>) -> Self {
        let line_inner = LineIdInner::WithBreak {
            line_number,
            line_content,
            line_break,
        };

        Self { line_inner }
    }

    /// Create a [`LineId`] from a line with no content, but with solely a line
    /// break.
    #[inline]
    pub const fn break_only(line_number: NonZero<usize>, line_break: LineBreak<'a, S>) -> Self {
        let line_inner = LineIdInner::BreakOnly { line_number, line_break };

        Self { line_inner }
    }
}

impl<'a, S> LineId<'a, S>
where
    S: SourceLines<'a> + ?Sized,
{
    /// Retrieve the line number of this line.
    #[inline]
    pub const fn number(&self) -> NonZero<usize> {
        let &Self { ref line_inner, .. } = self;

        line_inner.number()
    }

    /// Retrieve the [`contents`](LineContent) of this line.
    #[inline]
    pub const fn content(&self) -> Option<&LineContent<'a, S>> {
        let &Self { ref line_inner, .. } = self;

        line_inner.content()
    }

    /// Retrieve the [`line break`](LineBreak) of this line.
    #[inline]
    pub const fn content_break(&self) -> Option<&LineBreak<'a, S>> {
        let &Self { ref line_inner, .. } = self;

        line_inner.content_break()
    }

    /// Coerce this [`LineId`] to refer to a source with the same kind of lines.
    #[inline]
    pub fn coerce<O>(self) -> LineId<'a, O>
    where
        O: SourceLines<'a, Line = S::Line> + ?Sized,
    {
        self.map(|v| v)
    }

    /// Map the contents of this line to another type.
    #[inline]
    pub fn map<F, O>(self, map: F) -> LineId<'a, O>
    where
        F: Fn(S::Line) -> O::Line,
        O: SourceLines<'a> + ?Sized,
    {
        let Self { line_inner } = self;

        let line_inner = match line_inner {
            LineIdInner::Standalone { line_number, line_content } => {
                let line_content = line_content.map(map);

                LineIdInner::Standalone { line_number, line_content }
            }
            LineIdInner::WithBreak {
                line_number,
                line_content,
                line_break,
            } => {
                let line_content = line_content.map(&map);
                let line_break = line_break.map(map);

                LineIdInner::WithBreak {
                    line_number,
                    line_content,
                    line_break,
                }
            }
            LineIdInner::BreakOnly { line_number, line_break } => {
                let line_break = line_break.map(map);

                LineIdInner::BreakOnly { line_number, line_break }
            }
        };

        LineId { line_inner }
    }
}

/// An inner enumeration for the [`LineId`] type.
enum LineIdInner<'a, S>
where
    S: SourceLines<'a> + ?Sized,
{
    /// A standalone line, i.e, a line without a line break. Found in
    /// single-line sources.
    Standalone {
        line_number: NonZero<usize>,
        line_content: LineContent<'a, S>,
    },

    /// A line with content and a line break.
    WithBreak {
        line_number: NonZero<usize>,
        line_content: LineContent<'a, S>,
        line_break: LineBreak<'a, S>,
    },

    /// A line with no content, but with a line break.
    BreakOnly {
        line_number: NonZero<usize>,
        line_break: LineBreak<'a, S>,
    },
}

impl<'a, S> LineIdInner<'a, S>
where
    S: SourceLines<'a> + ?Sized,
{
    /// Retrieve the line number of this line.
    #[inline]
    const fn number(&self) -> NonZero<usize> {
        match self {
            &Self::Standalone { line_number, .. } | &Self::WithBreak { line_number, .. } | &Self::BreakOnly { line_number, .. } => {
                line_number
            }
        }
    }

    /// Retrieve the [`contents`](LineContent) of this line, if any.
    #[inline]
    const fn content(&self) -> Option<&LineContent<'a, S>> {
        match self {
            Self::Standalone { line_content, .. } | Self::WithBreak { line_content, .. } => Some(line_content),
            Self::BreakOnly { .. } => None,
        }
    }

    /// Retrieve the [`line break`](LineBreak) of this line, if any.
    #[inline]
    const fn content_break(&self) -> Option<&LineBreak<'a, S>> {
        match self {
            Self::Standalone { .. } => None,
            Self::WithBreak { line_break, .. } | Self::BreakOnly { line_break, .. } => Some(line_break),
        }
    }
}

impl<'a, S> hash::Hash for LineIdInner<'a, S>
where
    S: SourceLines<'a> + ?Sized,
    S::Line: hash::Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let (number, content, content_break) = (self.number(), self.content(), self.content_break());

        number.hash(state);
        content.hash(state);
        content_break.hash(state);
    }
}

impl<'a, S> Eq for LineIdInner<'a, S>
where
    S: SourceLines<'a> + ?Sized,
    S::Line: Eq,
{
}

impl<'a, S> PartialEq for LineIdInner<'a, S>
where
    S: SourceLines<'a> + ?Sized,
    S::Line: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Standalone {
                    line_number: l_line_number,
                    line_content: l_line_content,
                },
                Self::Standalone {
                    line_number: r_line_number,
                    line_content: r_line_content,
                },
            ) => l_line_number == r_line_number && l_line_content == r_line_content,
            (
                Self::WithBreak {
                    line_number: l_line_number,
                    line_content: l_line_content,
                    line_break: l_line_break,
                },
                Self::WithBreak {
                    line_number: r_line_number,
                    line_content: r_line_content,
                    line_break: r_line_break,
                },
            ) => l_line_number == r_line_number && l_line_content == r_line_content && l_line_break == r_line_break,
            (
                Self::BreakOnly {
                    line_number: l_line_number,
                    line_break: l_line_break,
                },
                Self::BreakOnly {
                    line_number: r_line_number,
                    line_break: r_line_break,
                },
            ) => l_line_number == r_line_number && l_line_break == r_line_break,
            _ => false,
        }
    }
}

impl<'a, S> Clone for LineIdInner<'a, S>
where
    S: SourceLines<'a> + ?Sized,
    S::Line: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::Standalone { line_number, line_content } => Self::Standalone {
                line_number: line_number.clone(),
                line_content: line_content.clone(),
            },
            Self::WithBreak {
                line_number,
                line_content,
                line_break,
            } => Self::WithBreak {
                line_number: line_number.clone(),
                line_content: line_content.clone(),
                line_break: line_break.clone(),
            },
            Self::BreakOnly { line_number, line_break } => Self::BreakOnly {
                line_number: line_number.clone(),
                line_break: line_break.clone(),
            },
        }
    }
}

impl<'a, S> fmt::Debug for LineIdInner<'a, S>
where
    S: SourceLines<'a> + ?Sized,
    S::Line: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Standalone { line_number, line_content } => f
                .debug_struct("Standalone")
                .field("line_number", line_number)
                .field("line_content", line_content)
                .finish(),
            Self::WithBreak {
                line_number,
                line_content,
                line_break,
            } => f
                .debug_struct("WithBreak")
                .field("line_number", line_number)
                .field("line_content", line_content)
                .field("line_break", line_break)
                .finish(),
            Self::BreakOnly { line_number, line_break } => f
                .debug_struct("BreakOnly")
                .field("line_number", line_number)
                .field("line_break", line_break)
                .finish(),
        }
    }
}

impl<'a, S> Copy for LineIdInner<'a, S>
where
    S: SourceLines<'a> + ?Sized,
    S::Line: Copy,
{
}
