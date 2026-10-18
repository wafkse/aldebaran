//! Assertion failures that preserve expected predicates and optional observed
//! components for source annotations. Built-in messages use character terminology
//! even for byte sources, and constructing a diagnostic does not validate its claim.

use core::{fmt, marker};

use aldebaran_baseline::span::Span;
use aldebaran_logic::{
    adhoc,
    prelude::{Assert, DefaultFormatter, Input},
};
use aldebaran_print::prelude::{Combine, Print};
use aldebaran_print::visualize::Visualize;
use aldebaran_report::prelude::{Annotated, Report, Simple, Title};
use aldebaran_source::prelude::{Component, SourceIter, SourceLines, SourceMetadata};

use crate::recovery::Recover;

use super::Cursor;

/// An expectation report pairing a predicate and optional observed component with
/// a diagnostic span. Callers establish the mismatch and source coordinates, since
/// construction validates neither and recovery with this report leaves the cursor unchanged.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// NOTE(invariant): None.
pub struct Expected<C, A>
where
    C: Input + Component,
    A: Assert<C>,
{
    /// Assertion that failed, with the component when input was present.
    expect: Expect<C, A>,

    /// Span used to annotate the mismatch or missing-input boundary.
    target: Span,
}

impl<C, A> fmt::Debug for Expected<C, A>
where
    C: Input + Component,
    A: Assert<C>,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { expect, target } = self;

        formatter
            .debug_struct("Expected")
            .field("expect", expect)
            .field("target", target)
            .finish()
    }
}

impl<C, A> Expected<C, A>
where
    C: Input + Component,
    A: Assert<C>,
{
    /// Report absent input at a caller-selected boundary without inspecting a source.
    #[inline]
    #[must_use]
    pub const fn this(subject: A, target: Span) -> Self {
        let expect = Expect::none(subject);

        Self { expect, target }
    }

    /// Report a caller-established mismatch without rechecking `subject` or the span.
    #[inline]
    #[must_use]
    pub const fn found(subject: A, exist: C, span: Span) -> Self {
        let expect = Expect::some(exist, subject);
        let target = span;

        Self { expect, target }
    }
}

impl<C, A> Report for Expected<C, A>
where
    C: Input + Component,
    A: Assert<C>,
{
    type Title = str;
    type Kind = Simple;
    type Annotations = Self;

    #[inline]
    fn title(&self) -> &Self::Title {
        let Self {
            expect: Expect { exist, .. },
            ..
        } = self;

        match exist {
            Some(_) => "unexpected character",
            None => "missing character",
        }
    }

    #[inline]
    fn kind(&self) -> &Self::Kind {
        &Simple::Error
    }

    #[inline]
    fn annotations(&self) -> &Self::Annotations {
        self
    }
}

impl<C, A> Print for Expected<C, A>
where
    C: Input + Component,
    A: Assert<C> + Print,
{
    type Context = ();
    #[inline]
    fn print_with_context<W>(&self, writer: &mut W, _: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        self.title().content().print(writer)
    }
}

impl<C, A> Annotated for Expected<C, A>
where
    C: Input + Component,
    A: Assert<C>,
{
    type Title = Expect<C, A>;

    #[inline]
    fn message(&self) -> &Self::Title {
        let Self { expect, .. } = self;

        expect
    }

    #[inline]
    fn target(&self) -> Span {
        let &Self { target, .. } = self;

        target
    }
}

/// An expectation message distinguishing missing input from a reported mismatch.
/// The predicate supplies its description without requiring a failed component to exist.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// NOTE(invariant): None.
pub struct Expect<C, A>
where
    C: Input + Component,
    A: Assert<C>,
{
    /// Mismatching component, or `None` when input was missing.
    exist: Option<C>,

    /// Assertion the component was expected to satisfy.
    subject: A,
}

impl<C, A> fmt::Debug for Expect<C, A>
where
    C: Input + Component,
    A: Assert<C>,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { exist, subject } = self;

        formatter
            .debug_struct("Expect")
            .field("exist", &exist.as_ref().map(Visualize::visual))
            .field("subject", &adhoc::Debugged::<A, C>::now(subject))
            .finish_non_exhaustive()
    }
}

impl<C, A> Expect<C, A>
where
    C: Input + Component,
    A: Assert<C>,
{
    /// Describe the predicate requirement when no component was available.
    #[inline]
    #[must_use]
    pub const fn none(subject: A) -> Self {
        Self { exist: None, subject }
    }

    /// Describe a reported mismatch without testing whether `subject` rejects `exist`.
    #[inline]
    #[must_use]
    pub const fn some(exist: C, subject: A) -> Self {
        Self {
            exist: Some(exist),
            subject,
        }
    }
}

impl<C, A> Print for Expect<C, A>
where
    C: Input + Component,
    A: Assert<C>,
{
    type Context = ();
    fn print_with_context<W>(&self, writer: &mut W, _: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        let Self { exist, subject } = self;

        match exist {
            Some(exist) => {
                '`'.print(writer)?;
                exist.visualize(writer)?;
                '`'.sequence(' ')
                    .sequence("was found instead of the expected")
                    .sequence(' ')
                    .print(writer)
            }
            None => "expected a character that satisfies".sequence(' ').print(writer),
        }?;

        '`'.print(writer)?;
        Assert::output::<W, DefaultFormatter>(subject, writer)?;
        '`'.print(writer)
    }
}

impl<'source, S, A> Recover<'source, S> for Expected<S::Component, A>
where
    S: SourceIter<'source> + SourceLines<'source> + SourceMetadata<'source> + ?Sized,
    A: Assert<S::Component>,
    S::Component: Input,
{
    #[inline]
    fn recover(self, _: &mut Cursor<'source, S>) -> Result<Self, Self> {
        Ok(self)
    }
}

/// An expectation report describing borrowed candidates through a caller-selected
/// assertion view. Callers establish rejection and source coordinates, since
/// construction validates neither and recovery with this report leaves the cursor unchanged.
// NOTE(invariant): None.
pub struct OneOf<'borrow, C, K, A, F>
where
    C: Input + Component,
    A: Assert<C>,
    F: Fn(&'borrow K) -> &'borrow A,
{
    /// Candidate values and the mismatch or missing-input state used for reporting.
    of_many: OfMany<'borrow, C, K, A, F>,

    /// Span used to annotate the candidate failure.
    target: Span,

    /// Associates the error with the assertion type used for its candidates.
    _marker: marker::PhantomData<fn() -> A>,
}

impl<'borrow, C, K, A, F> OneOf<'borrow, C, K, A, F>
where
    C: Input + Component,
    A: Assert<C>,
    F: Fn(&'borrow K) -> &'borrow A,
{
    /// Report a caller-established candidate mismatch at `span` without rechecking it.
    /// `predicate` selects each candidate's displayed assertion, including on later rendering.
    #[inline]
    #[must_use]
    pub const fn expected(target_list: &'borrow [K], target_component: C, span: Span, predicate: F) -> Self {
        let of_many = OfMany::Missmatch {
            target_list,
            target_component,
            predicate,
        };
        let target = span;
        let _marker = marker::PhantomData;

        Self { of_many, target, _marker }
    }

    /// Report missing input at a caller-selected span without inspecting a source.
    /// `predicate` selects each candidate's displayed assertion, including on later rendering.
    #[inline]
    #[must_use]
    pub const fn missed(target_list: &'borrow [K], target: Span, predicate: F) -> Self {
        let of_many = OfMany::Missed { target_list, predicate };
        let _marker = marker::PhantomData;

        Self { of_many, target, _marker }
    }

    /// Source coordinates used by this report's annotation, as supplied by the caller
    /// without source validation.
    #[inline]
    #[must_use]
    pub const fn target(&self) -> Span {
        let &Self { target, .. } = self;

        target
    }
}

impl<'borrow, C, K> OneOf<'borrow, C, K, K, fn(&K) -> &K>
where
    C: Input + Component,
    K: Assert<C>,
{
    /// A candidate that is itself an assertion needs no separate display projection.
    #[inline]
    const fn assertion(value: &K) -> &K {
        value
    }

    /// Report a caller-established mismatch with candidate assertions displayed directly.
    #[inline]
    #[must_use]
    pub const fn identified(target_list: &'borrow [K], target_component: C, span: Span) -> Self {
        Self::expected(target_list, target_component, span, Self::assertion as for<'a> fn(&'a K) -> &'a K)
    }

    /// Report absent input with candidate assertions displayed directly.
    #[inline]
    #[must_use]
    pub const fn nonidentified(target_list: &'borrow [K], target: Span) -> Self {
        Self::missed(target_list, target, Self::assertion as for<'a> fn(&'a K) -> &'a K)
    }
}

impl<'borrow, C, K, A, F> Report for OneOf<'borrow, C, K, A, F>
where
    C: Input + Component,
    A: Assert<C>,
    F: Fn(&'borrow K) -> &'borrow A,
{
    type Title = str;
    type Kind = Simple;
    type Annotations = Self;

    #[inline]
    fn kind(&self) -> &Self::Kind {
        &Simple::Error
    }

    #[inline]
    fn title(&self) -> &Self::Title {
        let Self { of_many, .. } = self;

        match of_many {
            OfMany::Missmatch { .. } => "unexpected character",
            OfMany::Missed { .. } => "missing character",
        }
    }

    #[inline]
    fn annotations(&self) -> &Self::Annotations {
        self
    }
}

impl<'borrow, C, K, A, F> Annotated for OneOf<'borrow, C, K, A, F>
where
    C: Input + Component,
    A: Assert<C>,
    F: Fn(&'borrow K) -> &'borrow A,
{
    type Title = OfMany<'borrow, C, K, A, F>;

    #[inline]
    fn message(&self) -> &Self::Title {
        let Self { of_many, .. } = self;

        of_many
    }

    #[inline]
    fn target(&self) -> Span {
        self.target()
    }
}

impl<'borrow, C, K, A, F> fmt::Debug for OneOf<'borrow, C, K, A, F>
where
    C: Input + Component + fmt::Debug,
    A: Assert<C> + Print,
    F: Fn(&'borrow K) -> &'borrow A + fmt::Debug,
    K: fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { of_many, target, .. } = self;

        formatter
            .debug_struct("OneOf")
            .field("of_many", of_many)
            .field("target", target)
            .finish()
    }
}

impl<'source, 'borrow, S, K, A, F> Recover<'source, S> for OneOf<'borrow, S::Component, K, A, F>
where
    S: SourceIter<'source> + SourceLines<'source> + SourceMetadata<'source> + ?Sized,
    S::Component: Input,
    A: Assert<S::Component>,
    F: Fn(&'borrow K) -> &'borrow A,
{
    #[inline]
    fn recover(self, _: &mut Cursor<'source, S>) -> Result<Self, Self> {
        Ok(self)
    }
}

/// A candidate expectation message with distinct absent-input and mismatch states.
/// Candidate order determines description order, without proving whether any candidate accepts.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OfMany<'borrow, C, K, A, F>
where
    C: Component + Input,
    A: Assert<C> + 'borrow,
    F: Fn(&'borrow K) -> &'borrow A,
{
    /// A component was present but matched no candidate.
    Missmatch {
        /// Borrowed alternatives in diagnostic display order.
        target_list: &'borrow [K],

        /// Component rejected by every candidate assertion.
        target_component: C,

        /// Selects the assertion displayed for each candidate.
        predicate: F,
    },

    /// No component was present where a candidate was expected.
    Missed {
        /// Borrowed alternatives in diagnostic display order.
        target_list: &'borrow [K],

        /// Selects the assertion displayed for each candidate.
        predicate: F,
    },
}

impl<'borrow, C, K, A, F> Print for OfMany<'borrow, C, K, A, F>
where
    C: Component + Input,
    A: Assert<C> + 'borrow,
    F: Fn(&'borrow K) -> &'borrow A,
{
    type Context = ();
    fn print_with_context<W>(&self, writer: &mut W, _: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        match self {
            &Self::Missed {
                target_list,
                ref predicate,
            } => {
                "expected one of: ".print(writer)?;

                let mut target_iter = target_list.iter();

                if let Some(target_first) = target_iter.next() {
                    let target_value = predicate(target_first);

                    '`'.print(writer)?;

                    Assert::output::<W, DefaultFormatter>(target_value, writer)?;

                    ('`').print(writer)?;

                    for target_value in target_iter {
                        let target_value = predicate(target_value);

                        ','.sequence(' ').sequence('`').print(writer)?;

                        Assert::output::<W, DefaultFormatter>(target_value, writer)?;

                        '`'.print(writer)?;
                    }
                }

                " after this sequence".print(writer)
            }
            &Self::Missmatch {
                target_list,
                target_component,
                ref predicate,
            } => {
                "expected one of: ".print(writer)?;

                let mut target_iter = target_list.iter();

                if let Some(target_first) = target_iter.next() {
                    let target_value = predicate(target_first);

                    '`'.print(writer)?;

                    Assert::output::<W, DefaultFormatter>(target_value, writer)?;

                    ('`').print(writer)?;

                    for target_value in target_iter {
                        let target_value = predicate(target_value);

                        ','.sequence(' ').sequence('`').print(writer)?;

                        Assert::output::<W, DefaultFormatter>(target_value, writer)?;

                        '`'.print(writer)?;
                    }
                }

                " but found: ".print(writer)?;

                '`'.print(writer)?;

                target_component.visualize(writer)?;

                '`'.print(writer)?;

                " instead after this sequence".print(writer)
            }
        }
    }
}

impl<'borrow, C, K, A, F> fmt::Debug for OfMany<'borrow, C, K, A, F>
where
    C: Component + Input,
    A: Assert<C> + Print + 'borrow,
    F: Fn(&'borrow K) -> &'borrow A,
    K: fmt::Debug,
    F: fmt::Debug,
    C: fmt::Debug,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missmatch {
                target_list,
                target_component,
                predicate,
            } => f
                .debug_struct("Missmatch")
                .field("target_list", target_list)
                .field("target_component", target_component)
                .field("predicate", predicate)
                .finish_non_exhaustive(),
            Self::Missed { target_list, predicate } => f
                .debug_struct("Missed")
                .field("target_list", target_list)
                .field("predicate", predicate)
                .finish_non_exhaustive(),
        }
    }
}
