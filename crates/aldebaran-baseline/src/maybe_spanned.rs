//! Optional extents for values that may not occupy source coordinates.
//!
//! [`MaybeSpanned`] accommodates empty values, while [`Spanned`] supplies a span
//! unconditionally. Built-in strings report UTF-8 byte coordinates and slices
//! report element coordinates, both starting at zero.

use core::num::NonZero;

use crate::{span::Span, spanned::Spanned};

/// A value whose nonempty coordinate extent may be available.
///
/// Implementations choose the coordinate system they report, while [`Span`]
/// requires a nonzero length in that system. Empty built-in strings and slices
/// therefore have no span.
pub trait MaybeSpanned {
    /// Return this value's nonempty span, if available.
    fn maybe_spanned(&self) -> Option<Span>;
}

/// A [`Spanned`] value always has an available span.
impl<T> MaybeSpanned for T
where
    T: Spanned,
{
    #[inline]
    fn maybe_spanned(&self) -> Option<Span> {
        Some(self.span())
    }
}

/// Nonempty strings use a zero-based span measured in UTF-8 bytes.
impl MaybeSpanned for str {
    fn maybe_spanned(&self) -> Option<Span> {
        NonZero::new(self.len()).map(|length| Span::new(usize::MIN, length))
    }
}

/// Nonempty slices use zero-based coordinates measured in elements.
impl<T> MaybeSpanned for [T] {
    fn maybe_spanned(&self) -> Option<Span> {
        NonZero::new(self.len()).map(|length| Span::new(usize::MIN, length))
    }
}

impl<T> MaybeSpanned for &[T] {
    fn maybe_spanned(&self) -> Option<Span> {
        <[T] as MaybeSpanned>::maybe_spanned(self)
    }
}

impl<T> MaybeSpanned for &mut [T] {
    fn maybe_spanned(&self) -> Option<Span> {
        <[T] as MaybeSpanned>::maybe_spanned(self)
    }
}

/// [`None`] has no span, while [`Some`] delegates to the contained value.
impl<T> MaybeSpanned for Option<T>
where
    T: MaybeSpanned,
{
    fn maybe_spanned(&self) -> Option<Span> {
        self.as_ref().and_then(MaybeSpanned::maybe_spanned)
    }
}
