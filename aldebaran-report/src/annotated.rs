//! This module contains the [`Annotated`] trait, which is used to represent an
//! annotated structure.
//!
//! Annotations are used to provide additional context to an error, such as help
//! messages, notes, and suggestions.

pub mod title;

pub mod label;

use alloc::boxed::Box;

use aldebaran_dsa::prelude::RefOneOrMore;
use aldebaran_span::span::Span;

/// A trait that represents an annotated structure.
///
/// This is the core component of a report, and is used to provide meaningful comments about the source text.
pub trait Annotated {
    /// The title of the annotation.
    type Title: title::Title + ?Sized;

    /// Retrieve the title of the annotation.
    ///
    /// See [`title::Title`] for more information.
    fn message(&self) -> &Self::Title;

    /// Retrieve the nonempty diagnostic span associated with the annotation.
    ///
    /// A unit span beginning at the source length denotes a boundary-focused
    /// diagnostic such as missing input at end of source.
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

/// A type that exposes zero or more diagnostic annotations without allocating.
pub trait Annotations {
    /// Annotation type exposed by this collection.
    type Annotation: Annotated;

    /// Borrow the available annotations as a primary value plus a related slice.
    fn list(&self) -> Option<RefOneOrMore<'_, Self::Annotation>>;
}

/// An annotation collection that statically guarantees a primary annotation.
pub trait PrimaryAnnotations: Annotations {
    /// Borrow the primary annotation.
    fn primary(&self) -> &Self::Annotation;
}

impl<T> Annotations for T
where
    T: Annotated,
{
    type Annotation = T;

    #[inline]
    fn list(&self) -> Option<RefOneOrMore<'_, Self::Annotation>> {
        Some(RefOneOrMore::single(self))
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

/// Inline storage for a primary annotation and a fixed related set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InlineAnnotations<A, const N: usize>(
    /// The primary annotation.
    A,
    /// Related annotations stored inline in declaration order.
    [A; N],
);

// NOTE(invariant): The primary annotation is always present and every related annotation is stored inline in declaration order.
impl<A, const N: usize> InlineAnnotations<A, N> {
    /// Construct a fixed annotation set.
    #[inline]
    #[must_use]
    pub const fn new(primary: A, related: [A; N]) -> Self {
        Self(primary, related)
    }

    /// Borrow the primary annotation.
    #[inline]
    #[must_use]
    pub const fn primary(&self) -> &A {
        let Self(target_primary, ..) = self;

        target_primary
    }

    /// Borrow the related annotation array.
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
    fn list(&self) -> Option<RefOneOrMore<'_, Self::Annotation>> {
        let Self(primary, related) = self;

        Some(RefOneOrMore::new(primary, related))
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
    use aldebaran_span::span::Span;

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
