//! Base source abstraction and source-plane coordinates.
//!
//! [`Source`] defines the component type and nonempty footprint of a borrowed
//! one-dimensional source. Additional capabilities for iteration, dissection,
//! metadata, and line access are layered through independent traits rather than
//! being required by the base abstraction.

use core::num::NonZero;

use aldebaran_span::prelude::Span;

use crate::{component::Component, terminate::Terminated};

/// Determine the default terminator for a source.
///
/// This is a very simple type alias that refers to the [`TerminatedBy`]
/// associated type of the [`Terminated`] trait.
///
/// If this is not used, the result will be inmmensely verbose.
///
/// [`TerminatedBy`]: Terminated::TerminatedBy
pub type DefaultTerminator<'a, S> = <S as Terminated<'a, S>>::TerminatedBy;

/// A trait for source sequences that are borrowed over a lifetime `'a`.
///
/// # What is a source?
///
/// In this case, a source is a one-dimensional plane of potentially
/// variable-sized [`components`](Self::Component).
///
/// This trait serves as an unified interface for all types that can be used as
/// an error source.
///
/// The semantics of a source are loosely defined, however, some aspects are set
/// in stone:
///
/// A source:
/// - is always borrowed over a lifetime `'a`.
/// - is always continous over a one-dimensional plane.
/// - is always terminated by an arbitrary, non-static sequence. Such sequences are defined by the [`Terminated`] trait.
///
/// This allows for a wide range of types to be used as sources, such as
/// strings, slices, and even custom types.
///
/// An unified interface for all types that can be used as an error source serve
/// as a way to abstract over the source type, thus allowing for a generic error
/// reporting system.
///
/// ## Examples
///
/// - A [`prim@str`] would be a source, where the components are [`char`]s.
/// - A `[T]`, where `T` is a type parameter, would be a source, where the components are `T`s (where `T` satisfies the requirements for
///   being a [`component`](Component)).
///
/// Other vivid examples include:
///
/// - A differential source, where the source itself represents a difference between two sequences or a code diff.
/// - A hexdump source, where the source itself represents a hexdump of a binary stream.
///
/// # Source planes
///
/// This term (`source plane`) is used to refer to the one-dimensional plane
/// that a source represents.
///
/// For any receiver of a source, the source requires to identify its own
/// [`span`](Span) over the source plane.
///
/// The source also requires itself to expose a
/// [`line based`](crate::line::SourceLines::Line) interface,
/// which is to be used for error reporting on line-by-line basis.
///
/// A segment of a source plane is always represented by a [`span`](Span), which
/// is a non-zero section of the source plane. Due to the explicit design of the
/// [`Span`] type, a source segment may never be empty.
///
/// ## Augmentative traits
///
/// This trait is a base trait for a wide range of other traits that are used to
/// represent diverse requirements for error sources.
///
/// Currently, the following traits are augmentative to this trait
/// (non-exhaustive):
///
/// - [`crate::line::SourceLines`]: adds a line-based interface to the source.
/// - [`crate::dissect::SourceDissect`]: adds a way to dissect the source into smaller parts.
/// - [`crate::iter::SourceIter`]: adds an iterator-based interface to the source.
/// - [`crate::metadata::SourceMetadata`]: incorporates metadata about the source.
pub trait Source<'a>
where
    Self: 'a,
{
    /// The associated component type for this source sequence.
    ///
    /// For example, a [`prim@str`] would have a [`prim@char`] as its component type.
    ///
    /// For the generic case, a `[T]`, where `T` is a type parameter, would have
    /// `T` as its component type.
    type Component: Component;

    /// The explicit footprint of the source over its source plane.
    ///
    /// In other words, this is the [`span`](Span) that represents the entire
    /// source.
    ///
    /// This is represented as an [`Option`] to allow for sources that may not
    /// have a footprint (i.e., empty sources) to be handled in an explicit
    /// manner.
    ///
    /// For an empty source, the footprint is [`None`].
    fn footprint(&self) -> Option<Span>;

    /// Determine the size of the source. May be `0` if the source is empty.
    ///
    /// # Remarks
    ///
    /// # Included-by-default
    ///
    /// This is simply a convenience method that refers to [`Source::footprint`]
    /// to calculate the size of the source.
    ///
    /// For cases where the associated computational complexity of
    /// [`Source::footprint`] is deemed excessive, this method may be
    /// overridden with a proper implementation.
    #[inline]
    fn size(&self) -> usize {
        self.footprint().as_ref().map(Span::length).map(NonZero::get).unwrap_or(usize::MIN)
    }

    /// A [`span`](Span) that refers to the rightmost part of the source.
    ///
    /// # Remarks
    ///
    /// If the source is empty, this is an [`unit span`](Span::unit).
    ///
    /// ## Included-by-default
    ///
    /// This method is provided by default, this method makes use of
    /// [`Source::footprint`] to calculate the end of the source.
    ///
    /// For cases where the inherent computational complexity of
    /// [`Source::footprint`] is deemed excessive, this method may be overridden
    /// with a proper implementation.
    #[inline]
    fn start(&self) -> Span {
        let span_start = self.footprint().as_ref().map(Span::start).unwrap_or(usize::MIN);

        Span::unit(span_start)
    }

    /// A [`span`](Span) that refers to the rightmost part of the source.
    ///
    /// # Remarks
    ///
    /// If the source is empty, this is an [`unit span`](Span::unit).
    ///
    /// ## Included-by-default
    ///
    /// This method is provided by default, this method makes use of
    /// [`Source::footprint`] to calculate the end of the source.
    ///
    /// For cases where the inherent computational complexity of
    /// [`Source::footprint`] is deemed excessive, this method may be overridden
    /// with a proper implementation.
    #[inline]
    fn end(&self) -> Span {
        let span_end = self.footprint().as_ref().map(Span::end).unwrap_or(0);

        Span::unit(span_end.saturating_sub(1))
    }
}

/// Blanket implementation for references to sources.
impl<'source, 'borrow, S> Source<'source> for &'borrow S
where
    'source: 'borrow,
    S: Source<'source> + ?Sized,
    &'borrow S: 'source,
{
    type Component = S::Component;

    #[inline]
    fn footprint(&self) -> Option<Span> {
        S::footprint(self)
    }
}
