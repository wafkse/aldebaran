//! A module that provides the [`Spanned`] trait.
//!
//! Refer to the [`Spanned`] trait for more information.

use crate::span::Span;

/// Trait for types that have a definite [`Span`].
pub trait Spanned {
    /// Determine the associated [`Span`] of this type.
    fn span(&self) -> Span;
}

/// Blanket implementation of [`Spanned`] for [`Span`].
impl Spanned for Span {
    #[inline]
    fn span(&self) -> Span {
        *self
    }
}
