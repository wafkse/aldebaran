//! Messages located within the source supplied to a diagnostic renderer.
//!
//! [`Annotated`] supplies a message and target, while [`Annotations`] exposes
//! the primary annotation followed by related context through [`RefNonempty`].
//! [`InlineAnnotations`] stores a fixed collection without allocating.

pub mod title;

pub mod label;

use alloc::boxed::Box;

use aldebaran_baseline::prelude::RefNonempty;
use aldebaran_baseline::span::Span;

/// A message paired with a source-coordinate target.
///
/// The span must use the coordinates of the source later paired with the report.
/// `str` sources use UTF-8 byte offsets, while other sources define their own
/// units. The trait does not associate a span with a source or validate bounds.
pub trait Annotated {
    /// Formatting contract for the message at this location.
    type Title: title::Title + ?Sized;

    /// Text attached to the target, independently of the report's heading.
    fn message(&self) -> &Self::Title;

    /// Nonempty location in the paired source's coordinate system.
    ///
    /// A unit span at the source length can represent missing input after EOF.
    fn target(&self) -> Span;
}

impl<T> Annotated for &T
where
    T: Annotated + ?Sized,
{
    type Title = T::Title;

    #[inline]
    fn message(&self) -> &Self::Title {
        (**self).message()
    }

    #[inline]
    fn target(&self) -> Span {
        (**self).target()
    }
}

impl<T> Annotated for Box<T>
where
    T: Annotated + ?Sized,
{
    type Title = T::Title;

    #[inline]
    fn message(&self) -> &Self::Title {
        T::message(self)
    }

    #[inline]
    fn target(&self) -> Span {
        T::target(self)
    }
}

/// Access to a diagnostic's primary location and ordered related context.
///
/// An unannotated diagnostic exposes no collection. A nonempty collection keeps
/// its primary annotation first and preserves the order of related annotations.
pub trait Annotations {
    /// Message and target shared by the collection's primary and related values.
    type Annotation: Annotated;

    /// Primary and related locations in presentation order, or `None` for an
    /// unannotated diagnostic.
    fn list(&self) -> Option<RefNonempty<'_, Self::Annotation>>;
}

/// Access to a primary annotation without handling an absent collection.
pub trait PrimaryAnnotations: Annotations {
    /// Location that identifies the diagnostic's primary concern.
    fn primary(&self) -> &Self::Annotation;
}

impl<T> Annotations for T
where
    T: Annotated,
{
    type Annotation = T;

    #[inline]
    fn list(&self) -> Option<RefNonempty<'_, Self::Annotation>> {
        Some(RefNonempty::single(self))
    }
}

impl<T> PrimaryAnnotations for T
where
    T: Annotated,
{
    #[inline]
    fn primary(&self) -> &Self::Annotation {
        self
    }
}

/// A primary annotation with a fixed amount of related context stored inline.
///
/// The related annotations retain their supplied presentation order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// NOTE(invariant): None.
pub struct InlineAnnotations<A, const N: usize>(
    /// Annotation identifying the diagnostic's primary concern.
    A,
    /// Related locations in the caller's presentation order.
    [A; N],
);

impl<A, const N: usize> InlineAnnotations<A, N> {
    /// Retain the primary annotation and related context in their supplied order.
    #[inline]
    #[must_use]
    pub const fn new(primary: A, related: [A; N]) -> Self {
        Self(primary, related)
    }

    /// Location identifying the diagnostic's primary concern.
    #[inline]
    #[must_use]
    pub const fn primary(&self) -> &A {
        let Self(target_primary, ..) = self;

        target_primary
    }

    /// Context following the primary annotation, in the caller's order.
    #[inline]
    #[must_use]
    pub const fn related(&self) -> &[A; N] {
        let Self(.., target_related) = self;

        target_related
    }
}

impl<A, const N: usize> Annotations for InlineAnnotations<A, N>
where
    A: Annotated,
{
    type Annotation = A;

    #[inline]
    fn list(&self) -> Option<RefNonempty<'_, Self::Annotation>> {
        let Self(primary, related) = self;

        Some(RefNonempty::multiple(primary, related))
    }
}

impl<A, const N: usize> PrimaryAnnotations for InlineAnnotations<A, N>
where
    A: Annotated,
{
    #[inline]
    fn primary(&self) -> &Self::Annotation {
        let Self(primary, ..) = self;

        primary
    }
}

#[cfg(test)]
mod tests {
    use aldebaran_baseline::span::Span;

    use super::{Annotations, InlineAnnotations, PrimaryAnnotations, label::Label};

    #[test]
    fn inline_annotations_preserve_primary_and_related_order() {
        let primary = Label::new("primary", Span::unit(0));
        let related = [
            Label::new("first related", Span::unit(1)),
            Label::new("second related", Span::unit(2)),
        ];
        let annotations = InlineAnnotations::new(primary, related);

        assert_eq!(annotations.primary(), &primary);
        assert_eq!(annotations.related(), &related);

        let list = annotations.list().expect("inline annotations are always nonempty");

        assert_eq!(list.first(), &primary);
        assert_eq!(list.rest(), &related);
    }

    #[test]
    fn primary_annotations_trait_matches_inline_primary_value() {
        let primary = Label::new("primary", Span::unit(3));
        let annotations = InlineAnnotations::new(primary, []);

        assert_eq!(PrimaryAnnotations::primary(&annotations), &primary);
    }
}
