//! Minimal inline annotations for diagnostic reports.
//!
//! [`Label`] pairs a printable message with one nonempty diagnostic span and
//! implements [`Annotated`]. It is the small building block
//! used by one-shot reports and fixed annotation collections.

use crate::prelude::{Annotated, Title};

use aldebaran_span::span::Span;

use aldebaran_source::prelude::Source;

/// A minimal inline annotation with one message and one nonempty diagnostic span.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Label<C>
where
    C: Title,
{
    /// Annotation message.
    message: C,

    /// Nonempty diagnostic span associated with the message.
    target: Span,
}

impl<C> Label<C>
where
    C: Title,
{
    /// Construct one span annotation.
    #[inline]
    #[must_use]
    pub const fn new(message: C, span: Span) -> Self {
        Self { message, target: span }
    }

    /// Instantiate a label that covers the available source text.
    ///
    /// Empty sources use [`Span::MIN`] as the unit boundary span at zero.
    #[inline]
    pub fn all<'a, S>(source: &'a S, message: C) -> Self
    where
        S: Source<'a> + ?Sized,
    {
        let target = match source.footprint() {
            Some(span) => span,
            None => Span::MIN,
        };

        Self { message, target }
    }
}

impl<C> Annotated for Label<C>
where
    C: Title,
{
    type Title = C;

    #[inline]
    fn message(&self) -> &Self::Title {
        let &Self { ref message, .. } = self;

        message
    }

    #[inline]
    fn target(&self) -> Span {
        let &Self { target, .. } = self;

        target
    }
}
