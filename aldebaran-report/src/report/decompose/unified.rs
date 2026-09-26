//! Unified borrowed title views for decomposed report values.
//!
//! [`UnifiedTitle`] represents either an ordinary report title or the title of an
//! inline annotation. This allows decomposition code to expose a unified title surface
//! without allocating or coercing distinct title types into strings.

use core::fmt;

use aldebaran_print::prelude::Print;

use crate::prelude::{Annotated, Title};

/// An abstract selection between a regular [`Title`] and a [`Annotated`]'s
/// [`title`](Annotated::Title).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UnifiedTitle<'a, T, A>
where
    T: Title + ?Sized,
    A: Annotated,
{
    /// The left-hand side possibility.
    Left(
        /// Borrowed ordinary report title.
        &'a T,
    ),

    /// The right-hand side possibility.
    Right(
        /// Borrowed annotation whose message supplies the title view.
        &'a A,
    ),
}

impl<'a, T, A> UnifiedTitle<'a, T, A>
where
    T: Title + ?Sized,
    A: Annotated,
{
    /// Create an unified title that simply uses the title.
    #[inline]
    pub const fn title(target_value: &'a T) -> Self {
        Self::Left(target_value)
    }

    /// Create an unified title that uses the annotated title.
    #[inline]
    pub const fn annotated(target_value: &'a A) -> Self {
        Self::Right(target_value)
    }

    /// Retrieve the title (left-hand side) of the unified title.
    #[inline]
    pub const fn left(&self) -> Option<&T> {
        match self {
            Self::Left(title) => Some(title),
            _ => None,
        }
    }

    /// Retrieve the annotated title (right-hand side) of the unified title.
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
    fn print_with_ctx<W>(&self, writer: &mut W, _: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        match self {
            Self::Left(title) => title.content().print(writer),
            Self::Right(annotated) => annotated.message().content().print(writer),
        }
    }
}
