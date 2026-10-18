//! Numbered source lines with separate content and terminator views.
//!
//! [`LineId`] represents an unterminated content line, content followed by a
//! break, or a break without content. Each shape has a nonempty combined span
//! even though either content or a terminator can be absent.

use core::{fmt, hash, num::NonZero};

use aldebaran_baseline::prelude::{Span, Spanned};

use crate::line::{LineBreak, LineContent, SourceLines};

/// A line's traversal number, content view, and optional terminator view.
///
/// The content and terminator together determine the line's span, allowing
/// consumers to distinguish visible text from the break while retaining their
/// combined extent. Its one-based number is assigned by the traversal that
/// produced the identity.
// NOTE(invariant): HasContent And HasBreak => Content.Source = Break.Source And Content.End = Break.Start
pub struct LineId<'a, S>
where
    S: SourceLines<'a> + ?Sized,
{
    /// The content and break shape associated with this line identity.
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
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
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
    /// Construct a line from its number, content, and optional terminator.
    ///
    /// When `line_break` is present, the caller must ensure its span immediately
    /// follows the content span and both refer to the same source.
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

    /// Represent an unterminated line with nonempty content.
    #[inline]
    pub const fn standalone(line_number: NonZero<usize>, line_content: LineContent<'a, S>) -> Self {
        let line_inner = LineIdInner::Standalone { line_number, line_content };

        Self { line_inner }
    }

    /// Represent nonempty content followed by a recognized break.
    /// The caller must provide adjacent spans from the same source.
    #[inline]
    pub const fn full(line_number: NonZero<usize>, line_content: LineContent<'a, S>, line_break: LineBreak<'a, S>) -> Self {
        let line_inner = LineIdInner::WithBreak {
            line_number,
            line_content,
            line_break,
        };

        Self { line_inner }
    }

    /// Represent a blank line whose nonempty extent consists of its break.
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
    /// One-based number used to label this line in its producing traversal.
    #[inline]
    pub const fn number(&self) -> NonZero<usize> {
        let &Self { ref line_inner, .. } = self;

        line_inner.number()
    }

    /// Renderable content, absent for a line consisting only of a break.
    #[inline]
    pub const fn content(&self) -> Option<&LineContent<'a, S>> {
        let &Self { ref line_inner, .. } = self;

        line_inner.content()
    }

    /// Recognized break, absent for an unterminated content line.
    #[inline]
    pub const fn content_break(&self) -> Option<&LineBreak<'a, S>> {
        let &Self { ref line_inner, .. } = self;

        line_inner.content_break()
    }

    /// Change the source type while preserving the line view and spans.
    #[inline]
    pub fn coerce<O>(self) -> LineId<'a, O>
    where
        O: SourceLines<'a, Line = S::Line> + ?Sized,
    {
        self.map(|v| v)
    }

    /// Adapt content and break representations without rebasing the line.
    /// Mapped views must preserve their original regions and adjacency.
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

/// Nonempty line shapes that distinguish absent content from an absent break.
enum LineIdInner<'a, S>
where
    S: SourceLines<'a> + ?Sized,
{
    /// Content with no recognized terminator.
    Standalone {
        /// One-based position assigned by the producing traversal.
        line_number: NonZero<usize>,

        /// The nonempty content extent before the line ends.
        line_content: LineContent<'a, S>,
    },

    /// Nonempty content followed by a recognized line break.
    WithBreak {
        /// One-based position assigned by the producing traversal.
        line_number: NonZero<usize>,

        /// The nonempty content extent before the line break.
        line_content: LineContent<'a, S>,

        /// The terminator view and its absolute source extent.
        line_break: LineBreak<'a, S>,
    },

    /// A line with no content, but with a line break.
    BreakOnly {
        /// One-based position assigned by the producing traversal.
        line_number: NonZero<usize>,

        /// The terminator view and its absolute source extent.
        line_break: LineBreak<'a, S>,
    },
}

impl<'a, S> LineIdInner<'a, S>
where
    S: SourceLines<'a> + ?Sized,
{
    /// Traversal position shared by all line shapes.
    #[inline]
    const fn number(&self) -> NonZero<usize> {
        match self {
            &Self::Standalone { line_number, .. } | &Self::WithBreak { line_number, .. } | &Self::BreakOnly { line_number, .. } => {
                line_number
            }
        }
    }

    /// Content extent, absent when the break occupies the entire line.
    #[inline]
    const fn content(&self) -> Option<&LineContent<'a, S>> {
        match self {
            Self::Standalone { line_content, .. } | Self::WithBreak { line_content, .. } => Some(line_content),
            Self::BreakOnly { .. } => None,
        }
    }

    /// Terminator extent, absent when the content reaches the source end.
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
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
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
                line_number: *line_number,
                line_content: line_content.clone(),
            },
            Self::WithBreak {
                line_number,
                line_content,
                line_break,
            } => Self::WithBreak {
                line_number: *line_number,
                line_content: line_content.clone(),
                line_break: line_break.clone(),
            },
            Self::BreakOnly { line_number, line_break } => Self::BreakOnly {
                line_number: *line_number,
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
