//! Compact source previews for debug and display output.
//!
//! [`Preview`] uses the source's default termination policy, while
//! [`PreviewWith`] selects a policy explicitly. Both visualize only the first
//! line's content and share the same formatting for debug and display.

use crate::dissect::SourceDissect;

use aldebaran_print::visualize::Visualize;
use core::fmt;

use crate::{
    iter::SourceIter,
    prelude::{Terminated, Terminator},
};

use super::{LineId, LineIter, SourceLines};

/// First-line content formatted inside `preview('...')` with a selected policy.
///
/// Components use [`Visualize`] without debug escaping. Empty input and a
/// first line consisting only of a break both format as `preview(<empty source>)`.
/// Later lines and the first line's terminator are omitted.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
// NOTE(invariant): None.
pub struct PreviewWith<'a, S, T = <S as Terminated<'a, S>>::TerminatedBy>(&'a S, T)
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + SourceDissect<'a, View<'a> = &'a S> + ?Sized;

impl<'a, S, T> PreviewWith<'a, S, T>
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + SourceDissect<'a, View<'a> = &'a S> + ?Sized,
{
    /// Preview the source using [`core::default::Default::default`] for `T`.
    #[inline]
    pub fn source(target_source: &'a S) -> Self
    where
        T: Default,
    {
        let target_terminator = T::default();

        Self(target_source, target_terminator)
    }

    /// Preview the source using the supplied terminator.
    #[inline]
    pub const fn source_with(target_source: &'a S, target_terminator: T) -> Self {
        Self(target_source, target_terminator)
    }
}

impl<'a, S, T> fmt::Debug for PreviewWith<'a, S, T>
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + SourceDissect<'a, View<'a> = &'a S> + ?Sized,
    T: Terminator<'a, S>,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let &Self(target_source, terminator) = self;

        let mut line_iter = LineIter::<'a, S, T>::new_at_with(target_source, usize::MIN, terminator);

        write!(f, "preview(")?;

        let first_line = line_iter.next();

        match first_line.as_ref().and_then(LineId::<'a>::content) {
            Some(target_content) => {
                let target_content: &'a S = target_content.value();

                write!(f, "'")?;

                for target_component in SourceIter::iter(target_content) {
                    target_component.visualize(f)?;
                }

                write!(f, "'")?;
            }
            None => {
                write!(f, "<empty source>")?;
            }
        }

        write!(f, ")")
    }
}

impl<'a, S, T> fmt::Display for PreviewWith<'a, S, T>
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + SourceDissect<'a, View<'a> = &'a S> + ?Sized,
    T: Terminator<'a, S>,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

/// A [`PreviewWith`] using the source's [`Terminated::DEFAULT_TERMINATOR`].
///
/// Debug and display both use the compact first-line visualization, including
/// the empty-content placeholder.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
// NOTE(invariant): None.
pub struct Preview<'a, S>(&'a S)
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + SourceDissect<'a, View<'a> = &'a S> + ?Sized;

impl<'a, S> Preview<'a, S>
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + SourceDissect<'a, View<'a> = &'a S> + ?Sized,
{
    /// Use the source's [`Terminated::DEFAULT_TERMINATOR`] when formatting the preview.
    #[inline]
    pub const fn source(target_source: &'a S) -> Self {
        Self(target_source)
    }
}

impl<'a, S> fmt::Display for Preview<'a, S>
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + SourceDissect<'a, View<'a> = &'a S> + ?Sized,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl<'a, S> fmt::Debug for Preview<'a, S>
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + SourceDissect<'a, View<'a> = &'a S> + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let &Self(target_source) = self;

        let target_value = PreviewWith(target_source, S::DEFAULT_TERMINATOR);

        fmt::Debug::fmt(&target_value, f)
    }
}
