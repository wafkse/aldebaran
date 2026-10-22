//! Line terminators and their incremental state machines.
//!
//! Terminators recognize line breaks as source components arrive, allowing
//! traversal to discover lines without a separate scan. Composition prioritizes
//! machines by arity, but does not look ahead, so this priority does not by
//! itself guarantee the longest matching break.

use crate::dissect::SourceDissect;

use core::num::NonZero;
use core::ops::ControlFlow;
use core::{cmp::Ordering, marker::PhantomData};

use aldebaran_baseline::prelude::Span;

use crate::component::Component;
use crate::line::{LineBreak, SourceLines};
use crate::source::Source;

/// A copyable state machine that recognizes line breaks one source component at a time.
///
/// After returning a break, a terminator must retain or reset its state so it
/// can recognize later breaks. The returned span must end at the current
/// component and cover a valid source region, since line traversal uses it to
/// derive line content.
pub trait Terminator<'a, S>: Copy + 'static
where
    S: SourceLines<'a> + ?Sized,
{
    /// Classify the current component as continuing content or a completed break.
    /// A break must lie within the current line and end at this component's
    /// exclusive endpoint.
    fn state(&mut self, ctx: &TerminatorContext<'a, S>) -> Outcome<'a, S>;
}

/// Recognition result whose break payload carries the recognized source view.
/// Continuing input has no payload because it has not completed a line break.
pub type Outcome<'a, S> = ControlFlow<LineBreak<'a, S>>;

/// The source component and source-coordinate span presented to a terminator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// NOTE(invariant): Source[Span] = Component
pub struct TerminatorContext<'a, S>
where
    S: Source<'a> + ?Sized,
{
    /// Original source used to obtain a view of a recognized break.
    source: &'a S,

    /// Current source component presented for break recognition.
    component: S::Component,

    /// Absolute component extent, with length equal to its source-unit width.
    span: Span,
}

impl<'a, S> TerminatorContext<'a, S>
where
    S: Source<'a> + ?Sized,
{
    /// Rebind this context to a source with the same component type.
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
    /// Store a component context. The caller must ensure the component and span
    /// describe that position in `source`.
    #[inline]
    pub const fn raw(source: &'a S, component: S::Component, span: Span) -> Self {
        Self { source, component, span }
    }
}

impl<'a, S> TerminatorContext<'a, S>
where
    S: Source<'a> + ?Sized,
{
    /// Source in whose coordinate plane a recognized break can be dissected.
    #[inline]
    pub const fn source(&self) -> &'a S {
        let &Self { source, .. } = self;

        source
    }

    /// Current recognition input, which may also complete a multi-component break.
    #[inline]
    pub const fn component(&self) -> S::Component {
        let &Self { component, .. } = self;

        component
    }

    /// Absolute component extent ending at the latest possible break endpoint.
    #[inline]
    pub const fn span(&self) -> Span {
        let &Self { span, .. } = self;

        span
    }
}

/// Source-selected termination policy for line operations without an explicit policy.
///
/// Its initial value is independent of [`Default`], allowing a source to choose
/// different recognition settings from a terminator type's general default.
pub trait Terminated<'a, S>
where
    S: SourceLines<'a> + ?Sized,
{
    /// The default terminator value for this source.
    const DEFAULT_TERMINATOR: Self::TerminatedBy;

    /// The terminator type used for this source.
    type TerminatedBy: Terminator<'a, S>;
}

/// Arity-aware composition for line terminators.
pub trait TerminatorCompose: TerminatorArity {
    /// Accept breaks from either policy with higher arity taking priority.
    ///
    /// Equal arities favor the left policy. Priority applies to each arriving
    /// component without lookahead, so it does not guarantee a longest match.
    /// The lower-priority policy misses components that complete a break in
    /// the higher-priority policy, which matters for stateful recognition.
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

/// Alternative break policies with arity-based precedence.
///
/// On a shared component the higher-arity policy has priority, favoring the
/// left policy on ties. The other policy receives only components that do not
/// complete a higher-priority break, so composition does not provide independent
/// traversal state for both policies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// NOTE(invariant): None.
pub struct TerminateEither<C, L, R>
where
    C: Component,
    L: TerminatorArity,
    R: TerminatorArity,
{
    /// Left recognition policy, preferred when both policies have equal arity.
    left: L,

    /// Right recognition policy, preferred only when its arity is greater.
    right: R,

    /// Component domain shared by the composed recognition policies.
    _marker: PhantomData<C>,
}

impl<C, L, R> TerminatorArity for TerminateEither<C, L, R>
where
    C: Component,
    L: TerminatorArity,
    R: TerminatorArity,
{
    /// Maximum component count required by either machine.
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
    /// Compose two terminators as an ordered pair.
    #[inline]
    pub const fn pair(left: L, right: R) -> Self {
        Self {
            left,
            right,
            _marker: PhantomData,
        }
    }

    /// Left policy with its current recognition state and tie-breaking priority.
    #[inline]
    pub const fn left(&self) -> &L {
        let &Self { ref left, .. } = self;

        left
    }

    /// Right policy with recognition state reflecting the components it received.
    #[inline]
    pub const fn right(&self) -> &R {
        let &Self { ref right, .. } = self;

        right
    }

    /// Return the two terminators that make up this composition.
    #[inline]
    pub fn partitioned(self) -> (L, R) {
        let Self { left, right, _marker } = self;

        (left, right)
    }
}

/// Break recognition by equality with a single source component.
///
/// Each matching occurrence forms a break spanning only that component.
/// Recognition retains no partial prefix between components.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
// NOTE(invariant): None.
pub struct Single<C>
where
    C: Component,
{
    /// Component whose occurrence terminates the current line.
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
    /// Treat `target_component` as a line break.
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

/// Break recognition for a consecutive pair of source components.
///
/// A partial prefix contributes its original source span to the eventual break.
/// For repeated prefixes, the latest matching first component is the candidate,
/// so an earlier mismatch cannot pull unrelated input into the break.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Two<C>
where
    C: Component,
{
    /// Pair to recognize with no pending prefix.
    Empty([C; 2]),

    /// Pending first component awaiting the pair's second component.
    First {
        /// Recognition pair in source order, retained across partial matches.
        target_storage: [C; 2],

        /// Absolute extent of the pending prefix in the traversed source.
        s0: Span,
    },
}

impl Two<char> {
    /// A line terminator identified by a carriage-return and a line-feed, in
    /// sequence.
    pub const CR_LF: Self = Self::tuple(('\r', '\n'));

    /// A line terminator identified by a line-feed followed by a carriage-return.
    pub const LF_CR: Self = Self::tuple(('\n', '\r'));
}

impl Two<u8> {
    /// A line terminator identified by a carriage-return and a line-feed, in
    /// sequence.
    pub const CR_LF: Self = Self::tuple((b'\r', b'\n'));

    /// A line terminator identified by a line-feed followed by a carriage-return.
    pub const LF_CR: Self = Self::tuple((b'\n', b'\r'));
}

impl<C> Two<C>
where
    C: Component,
{
    /// Match the supplied components in tuple order.
    #[inline]
    pub const fn tuple((s0, s1): (C, C)) -> Self {
        Self::Empty([s0, s1])
    }

    /// Match the supplied components in array order.
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

/// Reports the maximum component count a terminator may need to match.
///
/// Combinators use this value to prioritize state machines. It does not provide
/// lookahead, so longest-match behavior is not guaranteed. Implementations must
/// report their maximum accurately for the priority to work.
pub trait TerminatorArity {
    /// Maximum number of components this terminator can match.
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

/// LF and CRLF line boundaries for string and byte sources.
///
/// CRLF forms a combined break and a bare CR remains content. The same policy
/// works with Unicode scalar components and bytes without changing their
/// source-coordinate units.
#[derive(Debug, PartialEq, Copy, Eq, PartialOrd, Ord, Hash, Clone)]
// NOTE(invariant): None.
pub struct Standard<C>(TerminateEither<C, Two<C>, Single<C>>)
where
    C: Component;

impl Standard<char> {
    /// Recognize LF and CRLF in Unicode scalar input from an initial state.
    #[inline]
    pub const fn new() -> Self {
        let target_machine = TerminateEither::pair(Two::<char>::CR_LF, Single::<char>::LF);

        Self(target_machine)
    }
}

impl Standard<u8> {
    /// Recognize LF and CRLF bytes from an initial state.
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

/// Emits periodic breaks after counting source-component receptions.
///
/// The first break follows `N + 1` component receptions, with `N = 0` treated
/// as `1`. Later breaks are `max(N, 1)` receptions apart. Thus `Every<0>` and
/// `Every<1>` behave identically.
#[repr(transparent)]
#[derive(Debug, Copy, Clone, Hash)]
// NOTE(invariant): Count In {None} Union Some(1..=Max(N, 1))
pub struct Every<const N: usize>(Option<NonZero<usize>>);

impl<const N: usize> Every<N> {
    /// Component-count interval between later breaks, with zero treated as one.
    const FORCEFUL_MODULUS: usize = match N {
        0 => 1,
        _ => N,
    };

    /// Return the current count, or `None` before the first component is processed.
    ///
    /// After a break, the count resets to one for the next interval.
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
