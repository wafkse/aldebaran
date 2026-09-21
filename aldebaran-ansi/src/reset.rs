//! A module for generic resettable ANSI sequences.
//!
//! See the [`Resettable`] trait for more information.

use core::{fmt, marker::PhantomData};

use crate::prelude::AnsiSequence;

/// A trait that indicates that a determined [`AnsiSequence`]'s effects
/// can be reset.
pub trait Resettable<I>: AnsiSequence<I> {
    /// Reset the effects of the current [`AnsiSequence`].
    fn reset<W>(&self, writer: &mut W) -> fmt::Result
    where
        W: fmt::Write;
}

/// A wrapper over a [`resettable`](Resettable) [`ANSI sequence`](AnsiSequence).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Reset<R, I>(R, PhantomData<I>)
where
    R: Resettable<I>;

impl<R, I> Reset<R, I>
where
    R: Resettable<I>,
{
    /// Wrap a [`Reset`] over the target [`resettable`](Resettable) value.
    #[inline]
    pub const fn this(target_value: R) -> Self {
        Self(target_value, PhantomData)
    }

    /// Retrieve the inner value `R` from this [`Reset`].
    #[inline]
    pub fn into_inner(self) -> R {
        let Self(inner, ..) = self;

        inner
    }

    /// Retrieve a reference the resettable inner value `R` from this [`Reset`].
    #[inline]
    pub const fn resettable(&self) -> &R {
        let &Self(ref inner, ..) = self;

        inner
    }

    /// Retrieve a mutable reference the resettable inner value `R` from this [`Reset`].
    #[inline]
    pub const fn resettable_mut(&mut self) -> &mut R {
        let &mut Self(ref mut inner, ..) = self;

        inner
    }
}

impl<R, I> AnsiSequence<I> for Reset<R, I>
where
    R: Resettable<I>,
{
    #[inline]
    fn emit_with_input<W>(&self, writer: &mut W, _: I) -> fmt::Result
    where
        W: fmt::Write,
    {
        let &Self(ref target_value, ..) = self;

        R::reset(&target_value, writer)
    }
}

impl<R, I> Resettable<I> for &R
where
    R: Resettable<I>,
    for<'a> &'a R: AnsiSequence<I>,
{
    #[inline]
    fn reset<W>(&self, writer: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        Resettable::reset(*self, writer)
    }
}
