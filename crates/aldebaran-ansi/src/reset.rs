//! Deactivation of terminal settings through the sequence interface.
//!
//! [`Resettable`] defines the reset corresponding to a setting, and [`Reset`]
//! exposes it as an [`AnsiSequence`]. Reset codes clear settings rather than
//! recovering the terminal state that preceded activation.

use core::{fmt, marker::PhantomData};

use crate::prelude::AnsiSequence;

/// An [`AnsiSequence`] that can write a corresponding reset sequence.
pub trait Resettable<I>: AnsiSequence<I> {
    /// Write a reset sequence for this value.
    ///
    /// Because a reset code can clear more state than this value's sequence changed,
    /// it does not restore prior terminal settings. Writing can also fail partway
    /// through, leaving the sink partially updated.
    fn reset<W>(&self, writer: &mut W) -> fmt::Result
    where
        W: fmt::Write;
}

/// Deactivation of a setting wherever an [`AnsiSequence`] is expected.
///
/// Emission uses the wrapped value's reset and ignores per-emission input.
/// It inherits that reset's scope, which can include other terminal settings.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
// NOTE(invariant): None.
pub struct Reset<R, I>(R, PhantomData<I>)
where
    R: Resettable<I>;

impl<R, I> Reset<R, I>
where
    R: Resettable<I>,
{
    /// Wrap a value whose [`Resettable::reset`] operation should be emitted.
    #[inline]
    pub const fn this(target_value: R) -> Self {
        Self(target_value, PhantomData)
    }

    /// Recover the original setting, allowing its activation sequence to be used again.
    #[inline]
    pub fn unwrap(self) -> R {
        let Self(inner, ..) = self;

        inner
    }

    /// Setting whose deactivation determines the emitted sequence.
    #[inline]
    pub const fn resettable(&self) -> &R {
        let &Self(ref inner, ..) = self;

        inner
    }

    /// Change the setting that determines subsequent reset emission.
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

        R::reset(target_value, writer)
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
