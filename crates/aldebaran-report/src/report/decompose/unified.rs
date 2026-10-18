//! Unified borrowed title views for decomposed report values.
//!
//! [`UnifiedTitle`] represents either an ordinary report title or the title of an
//! inline annotation. This allows decomposition code to expose a unified title surface
//! without allocating or coercing distinct title types into strings.

use core::fmt;

use aldebaran_print::prelude::Print;

use crate::prelude::{Annotated, Title};

/// A borrowed report title or annotation message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UnifiedTitle<'a, T, A>
where
    T: Title + ?Sized,
    A: Annotated,
{
    /// A borrowed report title.
    Left(
        &'a T,
    ),

    /// A borrowed annotation.
    Right(
        &'a A,
    ),
}

impl<'a, T, A> UnifiedTitle<'a, T, A>
where
    T: Title + ?Sized,
    A: Annotated,
{
    #[inline]
    pub const fn title(target_value: &'a T) -> Self {
        Self::Left(target_value)
    }

    #[inline]
    pub const fn annotated(target_value: &'a A) -> Self {
        Self::Right(target_value)
    }

    #[inline]
    pub const fn left(&self) -> Option<&T> {
        match self {
            Self::Left(title) => Some(title),
            _ => None,
        }
    }

    #[inline]
    pub const fn right(&self) -> Option<&A> {
        match self {
            Self::Right(annotated) => Some(annotated),
            _ => None,
        }
    }
}

impl<'a, T, A> Print for UnifiedTitle<'a, T, A>
where
    T: Title,
    A: Annotated,
{
    type Context = ();
    #[inline]
    fn print_with_context<W>(&self, writer: &mut W, _: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        match self {
            Self::Left(title) => title.content().print(writer),
            Self::Right(annotated) => annotated.message().content().print(writer),
        }
    }
}
