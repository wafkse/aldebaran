//! Structured failures produced by assertion-driven text consumption.
//!
//! [`Expected`] represents one failed assertion and [`OneOf`] represents failure
//! among several candidates. Both retain nonempty diagnostic spans and implement
//! the reporting interfaces without reducing error identity to display strings.

use core::{fmt, marker};

use aldebaran_logic::{
    adhoc,
    prelude::{Assert, DefaultFormatter, Input},
};
use aldebaran_print::prelude::{Combine, Print};
use aldebaran_report::prelude::{Annotated, Report, Simple, Title};
use aldebaran_source::prelude::{Component, SourceIter, SourceLines, SourceMetadata};
use aldebaran_span::span::Span;
use aldebaran_visualize::visual::Visualize;

use crate::recovery::Recover;

use super::Text;

/// An error produced when one assertion cannot be satisfied.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Expected<C, A>
where
    C: Input + Component,
    A: Assert<C>,
{
    /// Expected assertion and optional mismatched component.
    expect: Expect<C, A>,

    /// Nonempty diagnostic span where the assertion failed.
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
    /// Construct a failure for a missing component at one diagnostic span.
    ///
    /// A unit span beginning at the source length represents the missing
    /// component boundary without a separate point type. Such a span begins at
    /// the exclusive source end and is intentionally not source-contained.
    #[inline]
    #[must_use]
    pub const fn this(subject: A, target: Span) -> Self {
        let expect = Expect::none(subject);

        Self { expect, target }
    }

    /// Construct a failure for one mismatched existing component.
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
    fn print_with_ctx<W>(&self, writer: &mut W, _: &Self::Context) -> fmt::Result
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

/// Printable expectation state for one assertion failure.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Expect<C, A>
where
    C: Input + Component,
    A: Assert<C>,
{
    /// Existing mismatched component when one was present.
    exist: Option<C>,

    /// Assertion that the source was expected to satisfy.
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
    /// Construct an expectation without an existing component.
    #[inline]
    #[must_use]
    pub const fn none(subject: A) -> Self {
        Self { exist: None, subject }
    }

    /// Construct an expectation with one mismatched component.
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
    fn print_with_ctx<W>(&self, writer: &mut W, _: &Self::Context) -> fmt::Result
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
    fn recover(self, _: &mut Text<'source, S>) -> Result<Self, Self> {
        Ok(self)
    }
}

/// Failure produced when none of several assertion candidates can match.
pub struct OneOf<'borrow, C, K, A, F>
where
    C: Input + Component,
    A: Assert<C>,
    F: Fn(&'borrow K) -> &'borrow A,
{
    /// Detailed candidate expectation state.
    of_many: OfMany<'borrow, C, K, A, F>,

    /// Nonempty diagnostic span where selection failed.
    target: Span,

    /// Assertion type marker.
    _marker: marker::PhantomData<fn() -> A>,
}

impl<'borrow, C, K, A, F> OneOf<'borrow, C, K, A, F>
where
    C: Input + Component,
    A: Assert<C>,
    F: Fn(&'borrow K) -> &'borrow A,
{
    /// Construct a candidate mismatch over an existing component.
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

    /// Construct a candidate failure for missing input at one diagnostic span.
    #[inline]
    #[must_use]
    pub const fn missed(target_list: &'borrow [K], target: Span, predicate: F) -> Self {
        let of_many = OfMany::Missed { target_list, predicate };
        let _marker = marker::PhantomData;

        Self { of_many, target, _marker }
    }

    /// Retrieve the diagnostic span of this candidate failure.
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
    /// Construct a direct candidate mismatch.
    #[inline]
    #[must_use]
    pub const fn identified(target_list: &'borrow [K], target_component: C, span: Span) -> Self {
        #[inline(always)]
        fn identity_ref<K>(value: &K) -> &K {
            value
        }

        Self::expected(target_list, target_component, span, identity_ref::<K> as _)
    }

    /// Construct a direct candidate failure for missing input.
    #[inline]
    #[must_use]
    pub const fn nonidentified(target_list: &'borrow [K], target: Span) -> Self {
        #[inline(always)]
        fn identity_ref<K>(value: &K) -> &K {
            value
        }

        Self::missed(target_list, target, identity_ref::<K> as _)
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
    fn recover(self, _: &mut Text<'source, S>) -> Result<Self, Self> {
        Ok(self)
    }
}

/// The title for an [`OneOf`] error.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OfMany<'borrow, C, K, A, F>
where
    C: Component + Input,
    A: Assert<C> + 'borrow,
    F: Fn(&'borrow K) -> &'borrow A,
{
    /// The title for an [`OneOf`] error when a component is found instead of
    /// the expected component.
    Missmatch {
        /// The list of `K` values that the [`Assert`] `A` pertains to.
        target_list: &'borrow [K],

        /// The component `C` that was found instead of the expected component,
        target_component: C,

        /// The predicate-mapping function `F`.
        predicate: F,
    },

    /// The title for an [`OneOf`] error when a component is missing rather than
    /// mismatched.
    Missed {
        /// The list of `K` values that the [`Assert`] `A` pertains to.
        target_list: &'borrow [K],

        /// The predicate-mapping function `F`.
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
    fn print_with_ctx<W>(&self, writer: &mut W, _: &Self::Context) -> fmt::Result
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
