//! Caller-owned conditions for enabling terminal presentation.
//!
//! [`Should`] binds a condition to borrowed context and implements
//! [`ContextualPredicate`]. Painting evaluates that condition when
//! [`only_when`](crate::brush::paint::Painted::only_when) is called, retaining
//! the result rather than the predicate or its context.

/// A reusable painting condition bound to caller-owned context.
///
/// Each [`holds`](Self::holds) evaluation consults the borrowed context. The
/// adapter retains no cached decision, leaving painting to snapshot the result
/// when the condition is applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// NOTE(invariant): None.
pub struct Should<'a, C, P>
where
    P: Fn(&'a C) -> bool,
{
    /// Borrowed context supplied to the predicate.
    context: &'a C,

    /// The condition evaluated against the borrowed context.
    predicate: P,
}

impl<'a, C, P> ContextualPredicate for Should<'a, C, P>
where
    P: Fn(&'a C) -> bool,
{
    type Context = C;

    #[inline]
    fn context(&self) -> &Self::Context {
        let &Self { context, .. } = self;

        context
    }

    #[inline]
    fn holds(&self) -> bool {
        let &Self {
            ref predicate, context, ..
        } = self;

        predicate(context)
    }
}

impl<'a, C, P> Should<'a, C, P>
where
    P: Fn(&'a C) -> bool,
{
    /// Bind the predicate to the context in an existing pair.
    #[inline]
    pub fn tuple((context, predicate): (&'a C, P)) -> Self {
        Self { context, predicate }
    }

    /// Bind the predicate to the borrowed context.
    #[inline]
    pub const fn satisfies(context: &'a C, predicate: P) -> Self {
        Self { context, predicate }
    }

    /// Evaluate the predicate against the stored context.
    #[inline]
    pub fn holds(&self) -> bool {
        let &Self { context, ref predicate } = self;

        predicate(context)
    }

    /// Caller-owned data against which the condition is evaluated.
    #[inline]
    pub const fn context(&self) -> &'a C {
        let &Self { context, .. } = self;

        context
    }

    /// Bound condition available for reuse without taking a new decision snapshot.
    #[inline]
    pub const fn predicate(&self) -> &P {
        let &Self { ref predicate, .. } = self;

        predicate
    }
}

impl<'a, C, P> Default for Should<'a, C, P>
where
    P: Fn(&'a C) -> bool,
    &'a C: Default,
    P: Default,
{
    #[inline]
    fn default() -> Self {
        let context = <&'a C as Default>::default();

        let predicate = P::default();

        Self { context, predicate }
    }
}

/// A presentation decision with separately accessible evaluation context.
/// Zero-argument closures satisfy this contract with unit context.
#[diagnostic::on_unimplemented(
    message = "{Self} is not a contextual predicate",
    label = "implementing this trait will enable the use of the `holds` method"
)]
pub trait ContextualPredicate {
    /// Data available to the condition, independent of a printable value's context.
    type Context;

    /// Evaluation data associated with this presentation condition.
    fn context(&self) -> &Self::Context;

    /// Evaluate whether the condition is satisfied.
    fn holds(&self) -> bool;
}

/// Zero-argument closures provide painting decisions without external context.
impl<F> ContextualPredicate for F
where
    F: Fn() -> bool,
{
    type Context = ();

    #[inline]
    fn context(&self) -> &Self::Context {
        &()
    }

    #[inline]
    fn holds(&self) -> bool {
        self()
    }
}
