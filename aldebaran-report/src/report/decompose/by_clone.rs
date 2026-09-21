//! Clone-based decomposition of composite report values.
//!
//! [`DecomposeByClone`] adapts a borrowed decomposable report by cloning the
//! components it yields. The adaptor is intended as temporary traversal state when
//! consumers require owned decomposition output but the source report is borrowed.

use core::marker::PhantomData;

use crate::prelude::Report;

/// Error decomposition through cloning.
///
/// This takes an error and makes it [`decomposable`] by cloning the
/// individual components.
///
/// This type can be indeed uaed as a standalone error, but it is not
/// recommended to do so, as it is meant to be used as a temporary value.
///
/// [`decomposable`]: super::Decompose
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DecomposeByClone<'a, E>
where
    E: Report<'a>,
{
    error: E,
    _marker: PhantomData<&'a ()>,
}

impl<'a, E> Report<'a> for DecomposeByClone<'a, E>
where
    E: Report<'a>,
{
    type Title = E::Title;

    type Kind = E::Kind;

    type Annotation = E::Annotation;

    type Source = E::Source;

    #[inline]
    fn kind(&self) -> &Self::Kind {
        let &Self { ref error, .. } = self;

        error.kind()
    }

    #[inline]
    fn title(&self) -> &Self::Title {
        let &Self { ref error, .. } = self;

        error.title()
    }

    #[inline]
    fn source(&self) -> &Self::Source {
        let &Self { ref error, .. } = self;

        error.source()
    }

    #[inline]
    fn annotations(&self) -> (&Self::Annotation, &[Self::Annotation]) {
        let &Self { ref error, .. } = self;

        error.annotations()
    }
}

impl<'a, E> DecomposeByClone<'a, E>
where
    E: Report<'a>,
{
    /// Makes an originally-undecomposable error decomposable by cloning the
    /// individual components.
    #[inline]
    pub const fn cloned(error: E) -> Self {
        Self {
            error,
            _marker: PhantomData,
        }
    }

    /// Retrieve the wrapped error.
    #[inline]
    pub const fn error(&self) -> &E {
        let &Self { ref error, .. } = self;

        error
    }
}
