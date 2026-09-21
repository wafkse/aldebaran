//! Line terminators and their incremental state machines.
//!
//! Terminators receive one source component at a time and decide when a line
//! break has been completed. Single and two-component machines can be composed,
//! with arity ensuring longer valid terminators take precedence over prefixes.

use core::num::NonZero;
use core::ops::ControlFlow;
use core::{cmp::Ordering, marker::PhantomData};

use aldebaran_span::prelude::Span;

use crate::component::Component;
use crate::line::{LineBreak, SourceLines};
use crate::source::Source;

/// Line terminators modeled as state machines.
///
/// This trait has a [`single method`](Terminator::state), which is responsible
/// for performing a singular state transition for the terminator.
///
/// Additionally, terminators must be trivially copyable, as they are expected
/// to be passed by value.
pub trait Terminator<'a, S>: Copy + 'static
where
    S: SourceLines<'a> + ?Sized,
{
    /// Perform a singular state transition.
    ///
    /// This will determine the [`Outcome`] of the transition.
    fn state(&mut self, ctx: &TerminatorContext<'a, S>) -> Outcome<'a, S>;
}

/// The outcome of a singular state transition for an arbitrary
/// [`Terminator`].
pub type Outcome<'a, S> = ControlFlow<LineBreak<'a, S>>;

/// An arbitrary transition context for a [`Terminator`], tied to a specific [`Source`] `S` and its [`composing type`] `C`.
///
/// [`composing type`]: Component
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TerminatorContext<'a, S>
where
    S: Source<'a> + ?Sized,
{
    /// The associated source for this transition context.
    source: &'a S,

    /// The component as input for this transition context.
    component: S::Component,

    /// The span of the aforementioned component.
    span: Span,
}

impl<'a, S> TerminatorContext<'a, S>
where
    S: Source<'a> + ?Sized,
{
    /// Perform a mapping operation for this transition context for `S` to some
    /// other transition context for `U`, given that their individual
    /// [`components`](Source::Component) match.
    #[inline]
    pub fn map<U, F>(&self, map: F) -> TerminatorContext<'a, U>
    where
        U: Source<'a, Component = S::Component> + ?Sized,
        F: FnOnce(&'a S) -> &'a U,
    {
        let &Self { source, component, span } = self;

        let source = map(source);

        TerminatorContext { source, component, span }
    }
}

impl<'a, S> TerminatorContext<'a, S>
where
    S: Source<'a> + ?Sized,
{
    /// Create a new transition context from its raw parts: a source, a
    /// component, and a span.
    #[inline]
    pub const fn from_raw_parts(source: &'a S, component: S::Component, span: Span) -> Self {
        Self { source, component, span }
    }
}

impl<'a, S> TerminatorContext<'a, S>
where
    S: Source<'a> + ?Sized,
{
    /// Retrieve the underlying [`source`](Source) of this transition context.
    #[inline]
    pub const fn source(&self) -> &'a S {
        let &Self { source, .. } = self;

        source
    }

    /// Retrieve the [`Component`] of this transition context.
    #[inline]
    pub const fn component(&self) -> S::Component {
        let &Self { component, .. } = self;

        component
    }

    /// Retrieve the [`Span`] of the component of this transition context.
    #[inline]
    pub const fn span(&self) -> Span {
        let &Self { span, .. } = self;

        span
    }
}

/// A trait for sequences that are terminated by an arbitrary sequence.
///
/// The [`terminator`](Self::TerminatedBy) is a type that models a state
/// machine.
pub trait Terminated<'a, S>
where
    S: SourceLines<'a> + ?Sized,
{
    /// The default, static [`terminator`](Terminator) for this sequence.
    const DEFAULT_TERMINATOR: Self::TerminatedBy;

    /// An associated type that represents a terminator suitable for this
    /// sequence.
    type TerminatedBy: Terminator<'a, S>;
}

/// A trait for combinable terminators.
///
/// This imposes an additional [`TerminatorArity`] constraint on the
/// terminator, as custom logic is required to determine the outcome of the
/// conjunction.
pub trait TerminatorCompose: TerminatorArity {
    /// Overlap the two terminators into a conjunction terminator.
    ///
    /// This effectively creates a new state machine that will evaluate the
    /// terminator with the highest arity first, and then the rest if the
    /// outcome is not conclusive.
    fn either<C, R>(self, right: R) -> TerminateEither<C, Self, R>
    where
        C: Component,
        Self: TerminatorArity + Sized,
        R: TerminatorArity;
}

impl<'a, T> TerminatorCompose for T
where
    T: TerminatorArity,
{
    #[inline]
    fn either<C, R>(self, right: R) -> TerminateEither<C, Self, R>
    where
        C: Component,
        Self: TerminatorArity,
        R: TerminatorArity,
    {
        TerminateEither::pair(self, right)
    }
}

/// A conjunction between two different terminators.
///
/// This imposes an additional [`TerminatorArity`] constraint on the
/// composing terminators, as custom logic is required to determine the
/// outcome of the conjunction.
///
/// See [`TerminatorCompose`] for the overall rationale for this requirement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TerminateEither<C, L, R>
where
    C: Component,
    L: TerminatorArity,
    R: TerminatorArity,
{
    /// The left hand side terminator.
    left: L,

    /// The right hand side terminator.
    right: R,

    /// A marker type to indicate an arbitrary component type.
    _marker: PhantomData<C>,
}

impl<C, L, R> TerminatorArity for TerminateEither<C, L, R>
where
    C: Component,
    L: TerminatorArity,
    R: TerminatorArity,
{
    /// A conjunction of the arity of the left and right hand side terminators.
    ///
    /// This is exactly the maximum of the two arities.
    const ARITY: usize = const { if L::ARITY > R::ARITY { L::ARITY } else { R::ARITY } };
}

impl<'a, S, L, R> Terminator<'a, S> for TerminateEither<S::Component, L, R>
where
    S: SourceLines<'a, Line = &'a S> + crate::dissect::SourceDissect<'a, View<'a> = &'a S> + ?Sized,
    L: Terminator<'a, S> + TerminatorArity,
    R: Terminator<'a, S> + TerminatorArity,
{
    #[inline]
    fn state(&mut self, ctx: &TerminatorContext<'a, S>) -> Outcome<'a, S> {
        let &mut Self {
            ref mut left,
            ref mut right,
            ..
        } = self;

        match L::ARITY.cmp(&R::ARITY) {
            Ordering::Greater | Ordering::Equal => {
                let (t0, t1) = (left, right);

                match t0.state(ctx) {
                    Outcome::Continue(_) => t1.state(ctx),
                    target_outcome => target_outcome,
                }
            }
            Ordering::Less => {
                let (t0, t1) = (right, left);

                match t0.state(ctx) {
                    Outcome::Continue(_) => t1.state(ctx),
                    target_outcome => target_outcome,
                }
            }
        }
    }
}

impl<C, L, R> TerminateEither<C, L, R>
where
    C: Component,
    L: TerminatorArity,
    R: TerminatorArity,
{
    /// Combine two terminators into a conjunction terminator from a 2 element
    /// pair.
    #[inline]
    pub const fn pair(left: L, right: R) -> Self {
        Self {
            left,
            right,
            _marker: PhantomData,
        }
    }

    /// Retrieve the left hand side terminator.
    #[inline]
    pub const fn left(&self) -> &L {
        let &Self { ref left, .. } = self;

        left
    }

    /// Retrieve the right hand side terminator.
    #[inline]
    pub const fn right(&self) -> &R {
        let &Self { ref right, .. } = self;

        right
    }

    /// Consume the conjunction terminator and return the composed terminators.
    #[inline]
    pub fn into_raw_parts(self) -> (L, R) {
        let Self { left, right, _marker } = self;

        (left, right)
    }
}

/// A terminator that represents a single [`component`](Component) `C`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Single<C>
where
    C: Component,
{
    /// The target component for this terminator.
    target_component: C,
}

impl Single<char> {
    /// A line-feed terminator for [`char`] components.
    pub const LF: Self = Self::one('\n');

    /// A carriage-return terminator for [`char`] components.
    pub const CR: Self = Self::one('\r');
}

impl Single<u8> {
    /// A line-feed terminator for [`u8`] components.
    pub const LF: Self = Self::one(b'\n');

    /// A carriage-return terminator for [`u8`] components.
    pub const CR: Self = Self::one(b'\r');
}

impl<C> Single<C>
where
    C: Component,
{
    /// Construct a new 1-ary terminator from a specific component.
    #[inline]
    pub const fn one(target_component: C) -> Self {
        Self { target_component }
    }
}

impl<'a, S> Terminator<'a, S> for Single<S::Component>
where
    S: SourceLines<'a, Line = &'a S> + crate::dissect::SourceDissect<'a, View<'a> = &'a S> + ?Sized,
{
    #[inline]
    fn state(&mut self, ctx: &TerminatorContext<'a, S>) -> Outcome<'a, S> {
        let &mut Self { target_component } = self;

        let target_value = ctx.component();

        if target_value == target_component {
            let span = ctx.span();

            let target_line = ctx.source().dissect(span);

            Outcome::Break(LineBreak::tuple((target_line, span)))
        } else {
            Outcome::Continue(())
        }
    }
}

/// A terminator that represents two [`component`](Component) `C`s.
///
/// This is a state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Two<C>
where
    C: Component,
{
    /// The initial state of the 2-ary terminator.
    Empty([C; 2]),

    /// The state where the first component has been matched.
    First {
        /// The storage for the two components to match.
        target_storage: [C; 2],
        /// The span of the first component.
        s0: Span,
    },
}

impl Two<char> {
    /// A line terminator identified by a carriage-return and a line-feed, in
    /// sequence.
    pub const CR_LF: Self = Self::tuple(('\r', '\n'));

    /// A line terminator identified by a line-feed and a carriage-return, in
    /// sequence.
    ///
    /// This type of line terminator is not as common as the reverse. But it
    /// still does appear in languages like Lua.
    pub const LF_CR: Self = Self::tuple(('\n', '\r'));
}

impl Two<u8> {
    /// A line terminator identified by a carriage-return and a line-feed, in
    /// sequence.
    pub const CR_LF: Self = Self::tuple((b'\r', b'\n'));

    /// A line terminator identified by a line-feed and a carriage-return, in
    /// sequence.
    ///
    /// This type of line terminator is not as common as the reverse. But it
    /// still does appear in languages like Lua.
    pub const LF_CR: Self = Self::tuple((b'\n', b'\r'));
}

impl<C> Two<C>
where
    C: Component,
{
    /// Construct a new 2-ary terminator from a 2-element tuple.
    #[inline]
    pub const fn tuple((s0, s1): (C, C)) -> Self {
        Self::Empty([s0, s1])
    }

    /// Construct a new 2-ary terminator from a 2-element array.
    #[inline]
    pub const fn array([c0, c1]: [C; 2]) -> Self {
        let target_value = [c0, c1];

        Self::Empty(target_value)
    }
}

impl<'a, S> Terminator<'a, S> for Two<S::Component>
where
    S: SourceLines<'a, Line = &'a S> + crate::dissect::SourceDissect<'a, View<'a> = &'a S> + ?Sized,
{
    #[inline]
    fn state(&mut self, ctx: &TerminatorContext<'a, S>) -> Outcome<'a, S> {
        let target_value = ctx.component();

        match self {
            &mut Self::Empty(target_storage) => {
                let [c0, ..] = target_storage;

                if target_value == c0 {
                    let s0 = ctx.span();

                    *self = Self::First { target_storage, s0 };

                    Outcome::Continue(())
                } else {
                    Outcome::Continue(())
                }
            }
            &mut Self::First { target_storage, s0 } => {
                let [_, c1] = target_storage;

                if target_value == c1 {
                    *self = Self::Empty(target_storage);

                    let s1 = ctx.span();

                    let span = s0.superset(s1);

                    let target_line = ctx.source().dissect(span);

                    Outcome::Break(LineBreak::tuple((target_line, span)))
                } else {
                    *self = Self::Empty(target_storage);

                    Outcome::Continue(())
                }
            }
        }
    }
}

/// An arity-describing trait for line terminators.
///
/// This is a requirement for terminator combinators, as the arity of each
/// respective terminator is crucial for the correct operation of the
/// combinator.
///
/// For example, the state machine can have a positive outcome if a larger
/// sequence of components has been matched, but not if a smaller sequence has
/// been matched, and, if any of the other terminators in the combinator have
/// a match case for the smaller sequence, it will result in incorrect
/// behavior.
///
/// In other words, this is required to match the longest possible sequence of
/// components as a terminator for a specific combinator.
///
/// A vivid example is the standard line terminator, `\r\n` and `\n`. Of course,
/// we would rather match `\r\n` than `\n`, as the former is more common. But
/// since state-machine transitions are done element-wise without any lookahead,
/// we need to know the arity of the terminator to make the correct decision.
pub trait TerminatorArity {
    /// The arity of the line terminator.
    ///
    /// In the context of this module, this is the number of components that a [`Terminator`] can potentially match.
    const ARITY: usize;
}

impl<C> TerminatorArity for Single<C>
where
    C: Component,
{
    const ARITY: usize = 1;
}

impl<C> TerminatorArity for Two<C>
where
    C: Component,
{
    const ARITY: usize = 2;
}

/// The standard terminator type.
///
/// This treats both `\r\n` and `\n` as line terminators.
#[derive(Debug, PartialEq, Copy, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct Standard<C>(TerminateEither<C, Two<C>, Single<C>>)
where
    C: Component;

impl Standard<char> {
    /// Create a new standard line terminator.
    ///
    /// See this type's documentation for more information.
    #[inline]
    pub const fn new() -> Self {
        let target_machine = TerminateEither::pair(Two::<char>::CR_LF, Single::<char>::LF);

        Self(target_machine)
    }
}

impl Standard<u8> {
    /// Create a new standard line terminator.
    ///
    /// See this type's documentation for more information.
    #[inline]
    pub const fn new() -> Self {
        let target_machine = TerminateEither::pair(Two::<u8>::CR_LF, Single::<u8>::LF);

        Self(target_machine)
    }
}

impl<'a, S> Terminator<'a, S> for Standard<S::Component>
where
    S: SourceLines<'a, Line = &'a S> + crate::dissect::SourceDissect<'a, View<'a> = &'a S> + ?Sized,
{
    #[inline]
    fn state(&mut self, ctx: &TerminatorContext<'a, S>) -> Outcome<'a, S> {
        let &mut Self(ref mut target_machine) = self;

        target_machine.state(ctx)
    }
}

/// A [`Terminator`] which indicates a line break every `max(N, 1)` [`Component`] receptions..
///
/// # Remarks
///
/// Due to intrinsic limitations of the [`Rem`] operator, [`Every<0>`] and [`Every<1>`] are identical in behavior.
///
/// [`Rem`]: core::ops::Rem
#[repr(transparent)]
#[derive(Debug, Copy, Clone, Hash)]
pub struct Every<const N: usize>(Option<NonZero<usize>>);

impl<const N: usize> Every<N> {
    /// A constant that evaluates `max(N, 1)`.
    const FORCEFUL_MODULUS: usize = match N {
        0 => 1,
        _ => N,
    };

    /// Determine the current count of this [`Every`] line terminator.
    #[inline]
    pub const fn count(&self) -> Option<NonZero<usize>> {
        let &Self(target_value) = self;

        target_value
    }
}

impl<'a, S, const N: usize> Terminator<'a, S> for Every<N>
where
    S: SourceLines<'a, Line = &'a S> + crate::dissect::SourceDissect<'a, View<'a> = &'a S> + ?Sized,
{
    #[inline]
    fn state(&mut self, ctx: &TerminatorContext<'a, S>) -> Outcome<'a, S> {
        match self {
            Self(None) => {
                *self = Every(Some(NonZero::<usize>::MIN));

                Outcome::Continue(())
            }
            Self(Some(count)) => match NonZero::new(count.get() % Self::FORCEFUL_MODULUS) {
                Some(..) => {
                    *count = count.saturating_add(1);

                    Outcome::Continue(())
                }
                None => {
                    *self = Every(Some(NonZero::<usize>::MIN));

                    let span = ctx.span();

                    let target_line = ctx.source().dissect(span);

                    Outcome::Break(LineBreak::tuple((target_line, span)))
                }
            },
        }
    }
}
