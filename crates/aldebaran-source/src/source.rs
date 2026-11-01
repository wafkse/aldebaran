//! Base source abstraction and source-plane coordinates.
//!
//! See [`Source`] for further information.

use core::num::NonZero;

use aldebaran_baseline::prelude::Span;

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
/// - is always continuous over a one-dimensional plane.
///
/// A source may also implement [`Terminated`] to define a line-termination
/// policy, and [`SourceLines`](crate::line::SourceLines) to expose line access.
/// These capabilities are layered on this base trait rather than required of
/// every source.
///
/// This allows for a wide range of types to be used as sources, such as
/// strings, slices, and even custom types.
///
/// A unified interface for all types that can be used as an error source serves
/// as a way to abstract over the source type, thus allowing for a generic error
/// reporting system.
///
/// ## Examples
///
/// - A [`prim@str`] would be a source, where the components are [`char`]s.
/// - A custom source over a `[T]` can use `T` as its component type, where `T`
///   satisfies the requirements for being a [`component`](Component).
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
/// For any receiver of a source, the source requires it to identify its own
/// [`span`](Span) over the source plane.
///
/// A [`SourceLines`](crate::line::SourceLines) implementation can expose a
/// line-based interface for error reporting. A segment of a source plane is
/// always represented by a [`span`](Span), which is a nonzero section of the
/// source plane. Due to the explicit design of the [`Span`] type, a source
/// segment may never be empty.

/// Component widths and spans use the same unit. Built-in strings use UTF-8
/// byte offsets even though iteration yields [`char`] values, whose widths may
/// differ.
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
    /// For example, a [`prim@str`] would have [`prim@char`] as its component
    /// type. A custom source backed by `[T]` can use `T` as its component type
    /// when `T` satisfies the requirements for being a [`component`](Component).
    type Component: Component;

    /// The explicit footprint of the source over its source plane.
    ///
    /// In other words, this is the [`span`](Span) that represents the entire
    /// source. This is represented as an [`Option`] to allow sources without a
    /// footprint (i.e., empty sources) to be handled explicitly.
    ///
    /// A nonempty footprint begins at coordinate zero, and its length equals
    /// the sum of the source components' widths from [`Component::size`].
    fn footprint(&self) -> Option<Span>;

    /// Determine the size of the source. It may be `0` if the source is empty.
    ///
    /// This is a convenience method that uses [`Source::footprint`] to
    /// calculate the size. If computing the footprint is expensive, an
    /// implementation may override this method. For strings, the size counts
    /// UTF-8 bytes rather than Unicode scalar values.
    #[inline]
    fn size(&self) -> usize {
        self.footprint().as_ref().map(Span::length).map(NonZero::get).unwrap_or(usize::MIN)
    }

    /// A [`span`](Span) that refers to the leftmost part of the source.
    ///
    /// If the source is empty, this is a [`unit span`](Span::unit) at zero.
    /// The default implementation derives this position from
    /// [`Source::footprint`], but can be overridden if that operation is
    /// expensive.
    #[inline]
    fn start(&self) -> Span {
        let span_start = self.footprint().as_ref().map(Span::start).unwrap_or(usize::MIN);

        Span::unit(span_start)
    }

    /// A [`span`](Span) that refers to the rightmost part of the source.
    ///
    /// If the source is empty, this is a [`unit span`](Span::unit). The default
    /// implementation derives this position from [`Source::footprint`], but
    /// can be overridden if that operation is expensive. For a multi-byte
    /// component, this anchor can fall inside the component rather than at its
    /// start.
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
