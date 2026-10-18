//! Source synchronization after a parse failure. Delimiter strategies preserve
//! the next synchronization component for the parser, while [`Skip`] discards a
//! bounded number of components.

use core::convert::Infallible;

use aldebaran_logic::prelude::{Not, OneOf, Ref};
use aldebaran_print::prelude::Print;

use aldebaran_source::prelude::{Component, Source, SourceDissect, SourceIter};

use crate::cursor::Cursor;

/// A recovery policy whose success and failure both retain the policy value.
/// Cursor movement is independent of the result and is not automatically rolled back.
pub trait Recover<'source, S>
where
    S: Source<'source> + SourceDissect<'source> + SourceIter<'source> + ?Sized,
{
    /// Attempt synchronization, retaining any cursor movement even on failure.
    fn recover(self, text: &mut Cursor<'source, S>) -> Result<Self, Self>
    where
        Self: Sized;
}

/// Synchronizes at the next occurrence of a component, leaving it unconsumed.
/// Absence of that component is also successful recovery at source exhaustion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// NOTE(invariant): None.
pub struct Until<C>(C)
where
    C: Component;

impl<C> Until<C>
where
    C: Component,
{
    /// Preserve the next occurrence of `value` for the parser after recovery.
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
    fn recover(self, text: &mut Cursor<'source, S>) -> Result<Self, Self>
    where
        Self: Sized,
    {
        let Self { 0: target_value } = self;

        text.ignore_while(Not::that(target_value));

        Ok(self)
    }
}

/// Synchronizes at the earliest source occurrence of any candidate, leaving it
/// unconsumed. If no candidate occurs, including for an empty set, recovery succeeds
/// at source exhaustion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// NOTE(invariant): None.
pub struct AnyOf<C, const N: usize>(OneOf<C, N>)
where
    C: Component;

impl<C, const N: usize> AnyOf<C, N>
where
    C: Component,
{
    /// Accept any listed delimiter as a synchronization point, regardless of array order.
    #[inline]
    pub const fn many(target_list: [C; N]) -> Self {
        Self(OneOf::these(target_list))
    }

    /// Delimiters preserved for the parser when encountered during recovery.
    /// Their array order does not affect which source position wins.
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
    fn recover(self, text: &mut Cursor<'source, S>) -> Result<Self, Self>
    where
        Self: Sized,
    {
        let Self { 0: ref value_set } = self;

        text.ignore_while(Not::that(Ref::that(value_set)));

        Ok(self)
    }
}

/// Discards up to `N` components, counting Unicode scalars for `str` rather than bytes.
/// Recovery succeeds even when fewer components remain, and `N = 0` preserves the position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Skip<const N: usize>;

impl<'source, S, const N: usize> Recover<'source, S> for Skip<N>
where
    S: Source<'source> + SourceDissect<'source> + SourceIter<'source> + ?Sized,
{
    #[inline]
    fn recover(self, text: &mut Cursor<'source, S>) -> Result<Self, Self>
    where
        Self: Sized,
    {
        for _ in 0..N {
            text.skip();
        }

        Ok(self)
    }
}

impl<'source, S> Recover<'source, S> for Infallible
where
    S: Source<'source> + SourceDissect<'source> + SourceIter<'source> + ?Sized,
{
    #[inline]
    fn recover(self, _: &mut Cursor<'source, S>) -> Result<Self, Self>
    where
        Self: Sized,
    {
        match self {}
    }
}

#[cfg(test)]
mod tests {
    use super::{AnyOf, Recover, Skip, Until};
    use crate::cursor::Cursor;

    #[test]
    fn until_leaves_the_delimiter_for_the_parser() {
        let mut text = Cursor::create("αβ;tail");
        let recovery = Until::that(';');

        assert_eq!(recovery.recover(&mut text), Ok(recovery));
        assert_eq!(text.index(), 4);
        assert_eq!(text.peek(), Some(';'));
        assert_eq!(recovery.recover(&mut text), Ok(recovery));
        assert_eq!(text.index(), 4);
    }

    #[test]
    fn absent_delimiter_stops_at_source_exhaustion() {
        let source = "αβ";
        let mut text = Cursor::create(source);
        let recovery = Until::that(';');

        assert_eq!(recovery.recover(&mut text), Ok(recovery));
        assert_eq!(text.index(), source.len());
        assert_eq!(text.peek(), None);
        assert_eq!(recovery.recover(&mut text), Ok(recovery));
        assert_eq!(text.index(), source.len());
    }

    #[test]
    fn any_of_stops_at_the_first_matching_delimiter() {
        let mut text = Cursor::create("α},tail");
        let recovery = AnyOf::many([',', '}']);

        assert_eq!(recovery.recover(&mut text), Ok(recovery));
        assert_eq!(text.index(), 2);
        assert_eq!(text.any(), Some('}'));
        assert_eq!(text.peek(), Some(','));
    }

    #[test]
    fn empty_delimiter_set_consumes_the_remaining_input() {
        let source = b"missing delimiter".as_slice();
        let mut text = Cursor::create(source);
        let recovery = AnyOf::<u8, 0>::many([]);

        assert_eq!(recovery.recover(&mut text), Ok(recovery));
        assert_eq!(text.index(), source.len());
        assert_eq!(text.peek(), None);
    }

    #[test]
    fn skip_counts_components_and_stops_at_exhaustion() {
        let mut text = Cursor::create("αβ;");

        assert_eq!(Skip::<0>.recover(&mut text), Ok(Skip::<0>));
        assert_eq!(text.index(), 0);
        assert_eq!(Skip::<2>.recover(&mut text), Ok(Skip::<2>));
        assert_eq!(text.index(), 4);
        assert_eq!(text.peek(), Some(';'));
        assert_eq!(Skip::<8>.recover(&mut text), Ok(Skip::<8>));
        assert_eq!(text.index(), 5);
        assert_eq!(text.peek(), None);
    }
}
