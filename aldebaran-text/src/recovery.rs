//! Text-based finite state machines.
//!
//! This module contains the core logic for the text-based finite state
//! machines, which are used to implement "dumb" lexers.
//!
//! Mainly used for error recovery.

use core::convert::Infallible;

use aldebaran_logic::prelude::{Not, OneOf, Ref};
use aldebaran_print::prelude::Print;

use aldebaran_source::prelude::{Component, Source, SourceDissect, SourceIter};

use crate::text::Text;

/// A state machine that incorporates error recovery from a
/// [`text-based`](crate::text::Text) input.
pub trait Recover<'source, S>
where
    S: Source<'source> + SourceDissect<'source> + SourceIter<'source> + ?Sized,
{
    /// Perform one error recovery step.
    ///
    /// Returns `Ok(Self)` if the recovery was successful, or `Err(Self)` if the
    /// recovery failed.
    fn recover(self, text: &mut Text<'source, S>) -> Result<Self, Self>
    where
        Self: Sized;
}

/// A [`Recover`] state machine that discards input until a certain character is
/// found.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Until<C>(C)
where
    C: Component;

impl<C> Until<C>
where
    C: Component,
{
    /// A new [`Until`] state machine formed from a target value.
    #[inline]
    pub const fn that(value: C) -> Self {
        Self(value)
    }
}

impl<'source, C, S> Recover<'source, S> for Until<C>
where
    C: Component,
    S: Source<'source, Component = C> + SourceDissect<'source> + SourceIter<'source> + ?Sized,
    C: Print,
    C::Context: Default,
{
    #[inline]
    fn recover(self, text: &mut Text<'source, S>) -> Result<Self, Self>
    where
        Self: Sized,
    {
        let Self { 0: target_value } = self;

        text.ignore_while(Not::that(target_value));

        Ok(self)
    }
}

/// A [`Recover`] state machine that discards input until one of a set of
/// matching characters is found.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AnyOf<C, const N: usize>(OneOf<C, N>)
where
    C: Component;

impl<C, const N: usize> AnyOf<C, N>
where
    C: Component,
{
    /// A new [`AnyOf`] state machine formed from a set of target values.
    #[inline]
    pub const fn many(target_list: [C; N]) -> Self {
        Self(OneOf::these(target_list))
    }

    /// Determine the storage array for this [`AnyOf`] state machine.
    #[inline]
    pub const fn storage(&self) -> &[C; N] {
        let &Self { 0: ref target_list } = self;

        target_list.storage()
    }
}

impl<'source, C, S, const N: usize> Recover<'source, S> for AnyOf<C, N>
where
    C: Component,
    S: Source<'source, Component = C> + SourceDissect<'source> + SourceIter<'source> + ?Sized,
    C: Print,
    C::Context: Default,
{
    #[inline]
    fn recover(self, text: &mut Text<'source, S>) -> Result<Self, Self>
    where
        Self: Sized,
    {
        let Self { 0: ref value_set } = self;

        text.ignore_while(Not::that(Ref::that(value_set)));

        Ok(self)
    }
}

/// A [`Recover`] state machine that discards input a certain number of times.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Skip<const N: usize>;

impl<'source, S, const N: usize> Recover<'source, S> for Skip<N>
where
    S: Source<'source> + SourceDissect<'source> + SourceIter<'source> + ?Sized,
{
    #[inline]
    fn recover(self, text: &mut Text<'source, S>) -> Result<Self, Self>
    where
        Self: Sized,
    {
        for _ in 0..N {
            let _ = text.skip();
        }

        Ok(self)
    }
}

impl<'source, S> Recover<'source, S> for Infallible
where
    S: Source<'source> + SourceDissect<'source> + SourceIter<'source> + ?Sized,
{
    #[inline]
    fn recover(self, _: &mut Text<'source, S>) -> Result<Self, Self>
    where
        Self: Sized,
    {
        match self {}
    }
}
