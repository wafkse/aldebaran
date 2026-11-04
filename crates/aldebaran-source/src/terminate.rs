//! Line terminators and their incremental state machines.
//!
//! See [`Terminator`], [`TerminatorCompose`], and [`TerminatorArity`] for further information.

use crate::dissect::SourceDissect;

use core::num::NonZero;
use core::ops::ControlFlow;
use core::{cmp::Ordering, marker::PhantomData};

use aldebaran_baseline::prelude::Span;

use crate::component::Component;
use crate::line::{LineBreak, SourceLines};
use crate::source::Source;

/// Line terminators modeled as state machines.
///
/// This trait has a [`single method`](Terminator::state), which is responsible
/// for performing a singular state transition for the terminator.
///
/// Additionally, terminators must be trivially copyable, as they are expected
/// to be passed by value. After returning a break, a terminator must retain or
/// reset its state so it can recognize later breaks. The returned span must end
/// at the current component and cover a valid source region, since line
/// traversal uses it to derive line content.
pub trait Terminator<'a, S>: Copy + 'static
where
    S: SourceLines<'a> + ?Sized,
{
    /// Perform a singular state transition.
    ///
    /// This determines the [`Outcome`] of the transition. A break must lie
    /// within the current line and end at the current component's exclusive
    /// endpoint.
    fn state(&mut self, ctx: &TerminatorContext<'a, S>) -> Outcome<'a, S>;
}

/// The outcome of a singular state transition for an arbitrary
/// [`Terminator`]. A completed break carries its source view in a [`LineBreak`].
pub type Outcome<'a, S> = ControlFlow<LineBreak<'a, S>>;

/// An arbitrary transition context for a [`Terminator`], tied to a specific
/// [`Source`] `S` and its [`composing type`](Component) `C`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// NOTE(invariant): Source[Span] = Component
pub struct TerminatorContext<'a, S>
where
    S: Source<'a> + ?Sized,
{
    /// The associated source for this transition context.
    source: &'a S,

    /// The component as input for this transition context.
    component: S::Component,

    /// The span of the aforementioned component, with a length equal to its
    /// source-unit width.
    span: Span,
}

impl<'a, S> TerminatorContext<'a, S>
where
    S: Source<'a> + ?Sized,
{
    /// Perform a mapping operation for this transition context for `S` to some
    /// other transition context for `U`, given that their individual
    /// [`components`](Source::Component) match.
    ///
    /// The returned context keeps the component and span, so the mapped source
    /// must contain that component at the same coordinate.
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
    ///
    /// The component and span must describe the same position in `source`.
    #[inline]
    pub const fn raw(source: &'a S, component: S::Component, span: Span) -> Self {
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
/// machine. Its initial value is independent of [`Default`], allowing a source
/// to choose different recognition settings from the terminator type's general
/// default.
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
/// This imposes an additional [`TerminatorArity`] constraint on the terminator,
/// as custom logic is required to determine the outcome of the combination.
pub trait TerminatorCompose: TerminatorArity {
    /// Overlap the two terminators into a combined terminator.
    ///
    /// This creates a new state machine that evaluates the terminator with the
    /// highest arity first, then tries the other if the first continues. Equal
    /// arities favor the left policy. Because each machine processes one component
    /// at a time without looking ahead, this priority does not guarantee the
    /// longest match. The lower-priority machine only receives components when
    /// the first machine continues, so it misses those that complete a break.
    fn either<C, R>(self, right: R) -> TerminateEither<C, Self, R>
    where
        C: Component,
        Self: TerminatorArity + Sized,
        R: TerminatorArity;
}

impl<T> TerminatorCompose for T
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

/// A combination of two different terminators.
///
/// The higher-arity policy is tried first, with the left policy preferred on a
/// tie. The other policy is tried only if the first continues, so it does not
/// receive components that completed a higher-priority break. Because the
/// policies share this input stream, they do not have independent traversal
/// state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// NOTE(invariant): None.
pub struct TerminateEither<C, L, R>
where
    C: Component,
    L: TerminatorArity,
    R: TerminatorArity,
{
    /// The left hand side terminator, preferred when the arities are equal.
    left: L,

    /// The right hand side terminator, preferred when it has greater arity.
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
    /// The arity of the composition, which is the maximum of the two arities.
    const ARITY: usize = const { if L::ARITY > R::ARITY { L::ARITY } else { R::ARITY } };
}

impl<'a, S, L, R> Terminator<'a, S> for TerminateEither<S::Component, L, R>
where
    S: SourceLines<'a, Line = &'a S> + SourceDissect<'a, View<'a> = &'a S> + ?Sized,
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
    /// Combine two terminators into a composed terminator from a 2-element
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

    /// Consume the composition and return its two terminators.
    #[inline]
    pub fn partitioned(self) -> (L, R) {
        let Self { left, right, _marker } = self;

        (left, right)
    }
}

/// A terminator that represents a single [`component`](Component) `C`.
///
/// Each matching occurrence forms a break spanning only that component, and
/// the terminator retains no partial prefix between components.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
// NOTE(invariant): None.
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
    S: SourceLines<'a, Line = &'a S> + SourceDissect<'a, View<'a> = &'a S> + ?Sized,
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
/// This is a state machine. A partial prefix contributes its original source
/// span to the eventual break. For repeated prefixes, the latest matching
/// first component is the candidate, so an earlier mismatch cannot pull
/// unrelated input into the break.
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
    S: SourceLines<'a, Line = &'a S> + SourceDissect<'a, View<'a> = &'a S> + ?Sized,
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
                let [c0, c1] = target_storage;

                if target_value == c1 {
                    *self = Self::Empty(target_storage);

                    let s1 = ctx.span();

                    let span = s0.superset(s1);

                    let target_line = ctx.source().dissect(span);

                    Outcome::Break(LineBreak::tuple((target_line, span)))
                } else {
                    *self = if target_value == c0 {
                        let s0 = ctx.span();

                        Self::First { target_storage, s0 }
                    } else {
                        Self::Empty(target_storage)
                    };

                    Outcome::Continue(())
                }
            }
        }
    }
}

/// An arity-describing trait for line terminators.
///
/// This is a requirement for terminator combinators, as the arity of each
/// terminator is crucial for their correct operation. For example, a larger
/// terminator may need to take priority over a shorter one, as with `\r\n` and
/// `\n` in the standard terminator. Since state-machine transitions happen
/// element-wise without lookahead, arity determines which machine is tried
/// first. It does not guarantee a longest match, and custom implementations
/// must report their maximum accurately for that priority to work.
pub trait TerminatorArity {
    /// The arity of the line terminator: the number of components that a
    /// [`Terminator`] can potentially match.
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
/// This treats both `\r\n` and `\n` as line terminators. CRLF forms a combined
/// break, while a bare CR remains content. The same policy works with Unicode
/// scalar components and bytes without changing their source-coordinate units.
#[derive(Debug, PartialEq, Copy, Eq, PartialOrd, Ord, Hash, Clone)]
// NOTE(invariant): None.
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

impl Default for Standard<char> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Default for Standard<u8> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, S> Terminator<'a, S> for Standard<S::Component>
where
    S: SourceLines<'a, Line = &'a S> + SourceDissect<'a, View<'a> = &'a S> + ?Sized,
{
    #[inline]
    fn state(&mut self, ctx: &TerminatorContext<'a, S>) -> Outcome<'a, S> {
        let &mut Self(ref mut target_machine) = self;

        target_machine.state(ctx)
    }
}

/// A [`Terminator`] which indicates a line break every `max(N, 1)`
/// [`Component`] receptions after its initial interval.
///
/// # Remarks
///
/// The first break follows `N + 1` component receptions, with `N = 0` treated
/// as `1`, while later breaks are `max(N, 1)` receptions apart. Due to the
/// behavior of the [`Rem`] operator, [`Every<0>`] and [`Every<1>`] are identical
/// in behavior.
///
/// [`Rem`]: core::ops::Rem
#[repr(transparent)]
#[derive(Debug, Copy, Clone, Hash)]
// NOTE(invariant): Count In {None} Union Some(1..=Max(N, 1))
pub struct Every<const N: usize>(Option<NonZero<usize>>);

impl<const N: usize> Every<N> {
    /// A constant that evaluates `max(N, 1)`.
    const FORCEFUL_MODULUS: usize = match N {
        0 => 1,
        _ => N,
    };

    /// Determine the current count of this [`Every`] line terminator.
    ///
    /// The count is `None` before the first component is processed and resets
    /// to one after a break.
    #[inline]
    pub const fn count(&self) -> Option<NonZero<usize>> {
        let &Self(target_value) = self;

        target_value
    }
}

impl<'a, S, const N: usize> Terminator<'a, S> for Every<N>
where
    S: SourceLines<'a, Line = &'a S> + SourceDissect<'a, View<'a> = &'a S> + ?Sized,
{
    #[inline]
    fn state(&mut self, ctx: &TerminatorContext<'a, S>) -> Outcome<'a, S> {
        match self {
            Self(None) => {
                *self = Every(Some(NonZero::<usize>::MIN));

                Outcome::Continue(())
            }
            Self(Some(count)) => {
                if NonZero::new(count.get() % Self::FORCEFUL_MODULUS).is_some() {
                    *count = count.saturating_add(1);

                    Outcome::Continue(())
                } else {
                    *self = Every(Some(NonZero::<usize>::MIN));

                    let span = ctx.span();

                    let target_line = ctx.source().dissect(span);

                    Outcome::Break(LineBreak::tuple((target_line, span)))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    //! Tests module for the [`terminate`] module.
    //!
    //! [`terminate`]: super

    use alloc::vec::Vec;
    use core::num::NonZero;

    use aldebaran_baseline::span::Span;

    use super::{Outcome, Terminator, TerminatorContext, Two};

    #[test]
    fn selected_lines_resume_the_terminator_state_at_the_selection_start() {
        use crate::line::{LineExtent, SourceLines};
        use aldebaran_baseline::prelude::Spanned;

        let source = "abcdefghi";
        let selection = source
            .segment_with(super::Every::<2>(None), Span::unit(4))
            .expect("selected periodic line");
        let lines: Vec<_> = selection.lines().map(|line| (line.number().get(), line.span().range())).collect();
        assert_eq!(lines, [(2, 3..5)]);
        assert_eq!(selection.metadata().bytes().range(), 3..5);
        assert_eq!(selection.metadata().length().partitioned(), (2, 2));
    }

    #[test]
    fn eof_selection_replays_saved_state_for_terminated_and_unterminated_lines() {
        use crate::line::{LineExtent, SourceLines};
        use aldebaran_baseline::prelude::Spanned;

        for (source, number, extent, content, line_break) in [("abcdefg", 3, 5..7, "f", Some("g")), ("abcdefgh", 4, 7..8, "h", None)] {
            let target = Span::unit(source.len());
            let selection = source.segment_with(super::Every::<2>(None), target).expect("final periodic line");
            let bytes = source
                .as_bytes()
                .segment_with(super::Every::<2>(None), target)
                .expect("final periodic byte line");

            assert_eq!(selection.metadata(), bytes.metadata());
            assert_eq!(selection.metadata().bytes().range(), extent);
            assert_eq!(selection.metadata().lines().range(), number..=number);
            assert_eq!(selection.metadata().length().partitioned(), (extent.len(), extent.len()));

            for _ in 0..2 {
                let mut lines = selection.lines();
                let line = lines.next().expect("selected periodic line");

                assert_eq!(line.number().get(), number);
                assert_eq!(line.span().range(), extent);
                assert_eq!(line.content().map(|content| *content.value()), Some(content));
                assert_eq!(line.content_break().map(|line_break| *line_break.line_break()), line_break);
                assert!(lines.next().is_none(), "only the final periodic line is selected");

                let mut lines = bytes.lines();
                let line = lines.next().expect("selected periodic byte line");

                assert_eq!(line.number().get(), number);
                assert_eq!(line.span().range(), extent);
                assert_eq!(line.content().map(|content| *content.value()), Some(content.as_bytes()));
                assert_eq!(
                    line.content_break().map(|line_break| *line_break.line_break()),
                    line_break.map(str::as_bytes)
                );
                assert!(lines.next().is_none(), "only the final periodic byte line is selected");
            }
        }
    }

    #[test]
    fn repeated_prefix_restarts_at_the_latest_utf8_component() {
        let source = "ααβ";
        let mut terminator = Two::tuple(('α', 'β'));
        let mut breaks = Vec::new();

        for (index, component) in source.char_indices() {
            let width = NonZero::new(component.len_utf8()).expect("a Unicode scalar has a nonzero UTF-8 width");
            let span = Span::new(index, width);
            let context = TerminatorContext::raw(source, component, span);

            match terminator.state(&context) {
                Outcome::Continue(()) => (),
                Outcome::Break(line_break) => breaks.push((*line_break.line_break(), line_break.span().range())),
            }
        }

        assert_eq!(breaks, [("αβ", 2..6)]);
    }

    #[test]
    fn mismatch_discards_a_prefix_without_spanning_unrelated_input() {
        let source = b"axbab".as_slice();
        let mut terminator = Two::tuple((b'a', b'b'));
        let mut breaks = Vec::new();

        for (index, component) in source.iter().copied().enumerate() {
            let span = Span::unit(index);
            let context = TerminatorContext::raw(source, component, span);

            match terminator.state(&context) {
                Outcome::Continue(()) => (),
                Outcome::Break(line_break) => breaks.push((*line_break.line_break(), line_break.span().range())),
            }
        }

        assert_eq!(breaks, [(b"ab".as_slice(), 3..5)]);
    }

    #[test]
    fn identical_components_complete_then_reset_for_the_next_pair() {
        let source = "aaaa";
        let mut terminator = Two::tuple(('a', 'a'));
        let mut breaks = Vec::new();

        for (index, component) in source.char_indices() {
            let span = Span::unit(index);
            let context = TerminatorContext::raw(source, component, span);

            match terminator.state(&context) {
                Outcome::Continue(()) => (),
                Outcome::Break(line_break) => breaks.push(line_break.span().range()),
            }
        }

        assert_eq!(breaks, [0..2, 2..4]);
    }
}
