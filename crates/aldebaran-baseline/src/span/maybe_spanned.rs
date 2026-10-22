//! Optional extents for values that may not occupy source coordinates.
//!
//! [`MaybeSpanned`] accommodates empty values, while [`Spanned`] supplies a span
//! unconditionally. Built-in strings report UTF-8 byte coordinates and slices
//! report element coordinates, both starting at zero.

use core::num::NonZero;

use super::{Span, Spanned};

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
    #[inline]
    fn maybe_spanned(&self) -> Option<Span> {
        NonZero::new(self.len()).map(|length| Span::new(usize::MIN, length))
    }
}

/// Nonempty slices use zero-based coordinates measured in elements.
impl<T> MaybeSpanned for [T] {
    #[inline]
    fn maybe_spanned(&self) -> Option<Span> {
        NonZero::new(self.len()).map(|length| Span::new(usize::MIN, length))
    }
}

impl<T> MaybeSpanned for &[T] {
    #[inline]
    fn maybe_spanned(&self) -> Option<Span> {
        <[T] as MaybeSpanned>::maybe_spanned(self)
    }
}

impl<T> MaybeSpanned for &mut [T] {
    #[inline]
    fn maybe_spanned(&self) -> Option<Span> {
        <[T] as MaybeSpanned>::maybe_spanned(self)
    }
}

/// [`None`] has no span, while [`Some`] delegates to the contained value.
impl<T> MaybeSpanned for Option<T>
where
    T: MaybeSpanned,
{
    #[inline]
    fn maybe_spanned(&self) -> Option<Span> {
        self.as_ref().and_then(MaybeSpanned::maybe_spanned)
    }
}

#[cfg(test)]
mod tests {
    use super::{MaybeSpanned, Span, Spanned};

    #[test]
    fn optional_spans_preserve_coordinates_and_absence() {
        let span = Span::unit(7);

        assert_eq!(span.span(), span);
        assert_eq!(span.maybe_spanned(), Some(span));
        assert_eq!(Some(span).maybe_spanned(), Some(span));
        assert_eq!(None::<Span>.maybe_spanned(), None);
        assert_eq!(Some(None::<Span>).maybe_spanned(), None);
        assert_eq!("".maybe_spanned(), None);
        assert_eq!("é🦀".maybe_spanned().map(|value| value.range()), Some(0..6));
    }

    #[test]
    fn slice_spans_count_elements_through_shared_and_mutable_references() {
        let mut values = ["é", "🦀"];
        let shared = values.as_slice();

        assert_eq!(<[&str]>::maybe_spanned(shared).map(|value| value.range()), Some(0..2));
        assert_eq!(<&[&str]>::maybe_spanned(&shared).map(|value| value.range()), Some(0..2));

        let exclusive = values.as_mut_slice();
        assert_eq!(<&mut [&str]>::maybe_spanned(&exclusive).map(|value| value.range()), Some(0..2));
        assert_eq!(exclusive[..0].maybe_spanned(), None);
    }
}
