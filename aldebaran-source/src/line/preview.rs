//! Compact previews for sources that support line segmentation.
//!
//! Preview wrappers render the first available source line through the source
//! component visualization rules. Callers may use the source default terminator
//! or provide an explicit terminator without allocating a temporary string.

use aldebaran_visualize::visual::Visualize;
use core::fmt;

use crate::{
    iter::SourceIter,
    prelude::{Terminated, Terminator},
};

use super::{LineId, LineIter, SourceLines};

/// A type that acts as a preview into the source `S`, where `S` is a source that is both segmented into lines and iterable over its
/// components.
///
/// This implements both [`fmt::Display`] and [`fmt::Debug`], where the first line is shown as their respective implementations.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PreviewWith<'a, S, T = <S as Terminated<'a, S>>::TerminatedBy>(&'a S, T)
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + crate::dissect::SourceDissect<'a, View<'a> = &'a S> + ?Sized;

impl<'a, S, T> PreviewWith<'a, S, T>
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + crate::dissect::SourceDissect<'a, View<'a> = &'a S> + ?Sized,
{
    /// Preview the target source `S` using the default terminator `T`.
    #[inline]
    pub fn source(target_source: &'a S) -> Self
    where
        T: Default,
    {
        let target_terminator = T::default();

        Self(target_source, target_terminator)
    }

    /// Preview the target source `S` using the target terminator `T`.
    #[inline]
    pub const fn source_with(target_source: &'a S, target_terminator: T) -> Self {
        Self(target_source, target_terminator)
    }
}

impl<'a, S, T> fmt::Debug for PreviewWith<'a, S, T>
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + crate::dissect::SourceDissect<'a, View<'a> = &'a S> + ?Sized,
    T: Terminator<'a, S>,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let &Self(target_source, terminator) = self;

        let mut line_iter = LineIter::<'a, S, T>::new_at_with(target_source, usize::MIN, terminator);

        write!(f, "preview(")?;

        let first_line = line_iter.next();

        match first_line.as_ref().map(LineId::<'a>::content).flatten() {
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
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + crate::dissect::SourceDissect<'a, View<'a> = &'a S> + ?Sized,
    T: Terminator<'a, S>,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

/// A type that acts as a preview into the source `S`, where `S` is a source that is both segmented into lines and iterable over its
/// components.
///
/// This implements both [`fmt::Display`] and [`fmt::Debug`], where the first line is shown as their respective implementations.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Preview<'a, S>(&'a S)
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + crate::dissect::SourceDissect<'a, View<'a> = &'a S> + ?Sized;

impl<'a, S> Preview<'a, S>
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + crate::dissect::SourceDissect<'a, View<'a> = &'a S> + ?Sized,
{
    /// Preview the target source `S`.
    #[inline]
    pub const fn source(target_source: &'a S) -> Self {
        Self(target_source)
    }
}

impl<'a, S> fmt::Display for Preview<'a, S>
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + crate::dissect::SourceDissect<'a, View<'a> = &'a S> + ?Sized,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl<'a, S> fmt::Debug for Preview<'a, S>
where
    S: SourceLines<'a, Line = &'a S> + SourceIter<'a> + crate::dissect::SourceDissect<'a, View<'a> = &'a S> + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let &Self(target_source) = self;

        let target_terminator = S::DEFAULT_TERMINATOR;

        let target_value = PreviewWith(target_source, target_terminator);

        fmt::Debug::fmt(&target_value, f)
    }
}
