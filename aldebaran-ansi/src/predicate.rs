//! Predicates for conditional logic during the painting process.
//!
//! See the [`ContextualPredicate`] trait for more information.

/// A predicate that holds an inherent context.
///
/// This is a struct that holds a context and a predicate, and can be used as an
/// [`ContextualPredicate`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Should<'a, C, P>
where
    P: Fn(&'a C) -> bool,
{
    context: &'a C,
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
    /// Instantiate a new contextual predicate from the given
    /// context and predicate 2-tuple.
    #[inline]
    pub fn tuple((context, predicate): (&'a C, P)) -> Self {
        Self { context, predicate }
    }

    /// Instantiate a new contextual predicate from the given
    /// context and predicate.
    #[inline]
    pub const fn satisfies(context: &'a C, predicate: P) -> Self {
        Self { context, predicate }
    }

    /// Determine if the predicate holds for the given context.
    #[inline]
    pub fn holds(&self) -> bool {
        let &Self { context, ref predicate } = self;

        predicate(context)
    }

    /// Retrieve the context for this contextual predicate.
    #[inline]
    pub const fn context(&self) -> &'a C {
        let &Self { context, .. } = self;

        context
    }

    /// Retrieve the predicate for this contextual predicate.
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

/// A trait for types that can be used as contextual predicates.
///
/// A contextual predicate is simply a predicate that holds an inherent context.
#[diagnostic::on_unimplemented(
    message = "{Self} is not a contextual predicate",
    label = "implementing this trait will enable the use of the `holds` method"
)]
pub trait ContextualPredicate {
    /// The context type for the predicate.
    type Context;

    /// Retrieve the context for this contextual predicate.
    fn context(&self) -> &Self::Context;

    /// Determine if the contextua; predicate holds.
    fn holds(&self) -> bool;
}

/// Blanket implementation for all types that implement the `Fn` trait.
///
/// Uses a unit (`()`) context.
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
