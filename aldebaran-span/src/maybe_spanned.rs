//! A module that provides the [`MaybeSpanned`] trait.
//!
//! Refer to the [`MaybeSpanned`] trait for more information.

use core::num::NonZero;

use crate::{span::Span, spanned::Spanned};

/// Trait for types that may have a span.
///
/// This is useful for types that may or may not have a span, and we want to
/// retrieve it if it exists.
///
/// This is the case for empty strings, for example.
pub trait MaybeSpanned {
    /// Retrieve the span of this type, if it exists.
    fn maybe_spanned(&self) -> Option<Span>;
}

/// If a type is [`Spanned`], it can be [`MaybeSpanned`].
impl<T> MaybeSpanned for T
where
    T: Spanned,
{
    #[inline]
    fn maybe_spanned(&self) -> Option<Span> {
        Some(self.span())
    }
}

impl MaybeSpanned for str {
    fn maybe_spanned(&self) -> Option<Span> {
        NonZero::new(self.len()).map(|length| Span::new(usize::MIN, length))
    }
}

/// Blanket implementation for all slice types.
impl<T> MaybeSpanned for [T] {
    fn maybe_spanned(&self) -> Option<Span> {
        NonZero::new(self.len()).map(|length| Span::new(usize::MIN, length))
    }
}

/// Blanket implementation for all slice types (references).
impl<T> MaybeSpanned for &[T] {
    fn maybe_spanned(&self) -> Option<Span> {
        <[T] as MaybeSpanned>::maybe_spanned(self)
    }
}

/// Blanket implementation for all slice types (mutable references).
impl<T> MaybeSpanned for &mut [T] {
    fn maybe_spanned(&self) -> Option<Span> {
        <[T] as MaybeSpanned>::maybe_spanned(self)
    }
}

/// Blanket implementation for [`Option`]s.
impl<T> MaybeSpanned for Option<T>
where
    T: MaybeSpanned,
{
    fn maybe_spanned(&self) -> Option<Span> {
        self.as_ref().and_then(MaybeSpanned::maybe_spanned)
    }
}
