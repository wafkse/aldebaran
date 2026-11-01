//! A module that provides the [`MaybeSpanned`] trait.
//!
//! See [`MaybeSpanned`] for further information.
//!
//! Empty strings have no span, while nonempty strings use UTF-8 byte coordinates
//! and slices use element coordinates, both starting at zero.

use core::num::NonZero;

use super::{Span, Spanned};

/// Trait for types that may have a span.
///
/// This is useful for types that may or may not have a span, and we want to
/// retrieve it if it exists. Empty strings and slices are examples.
pub trait MaybeSpanned {
    /// Retrieve the span of this type, if it exists.
    fn maybe_spanned(&self) -> Option<Span>;
}

/// If a type is [`Spanned`], it can also be [`MaybeSpanned`].
impl<T> MaybeSpanned for T
where
    T: Spanned,
{
    #[inline]
    fn maybe_spanned(&self) -> Option<Span> {
        Some(self.span())
    }
}

/// A nonempty string has a span measured in UTF-8 bytes from zero.
impl MaybeSpanned for str {
    #[inline]
    fn maybe_spanned(&self) -> Option<Span> {
        NonZero::new(self.len()).map(|length| Span::new(usize::MIN, length))
    }
}

/// A nonempty slice has a span measured in elements from zero.
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
    //! Tests module for the [`maybe_spanned`] module.
    //!
    //! [`maybe_spanned`]: super

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
