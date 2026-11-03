//! Incremental token streams backed directly by lexical source consumption.
//!
//! See [`LexStream`] for further information.

use core::fmt;

use aldebaran_grammar::stream::TokenStream;
use aldebaran_source::prelude::{Source, SourceDissect, SourceIter};

use super::{consume::Lex, internment::Internment};

/// A lexical token stream over one source cursor and persistent mutable context.
///
/// Each successful stream step recognizes and constructs exactly one lexical
/// value through `L`. The wrapped [`Internment`] guarantees that every step sees
/// the same construction context and advances the same source cursor.
pub struct LexStream<'source, 'context, S, L>(pub Internment<'source, 'context, S, L::Context>)
where
    S: Source<'source> + SourceDissect<'source> + SourceIter<'source> + ?Sized,
    L: Lex<'source, S>;

impl<'source, 'context, S, L> LexStream<'source, 'context, S, L>
where
    S: Source<'source> + SourceDissect<'source> + SourceIter<'source> + ?Sized,
    L: Lex<'source, S>,
{
    /// Create a lexical stream over one source and mutable lexical context.
    #[inline]
    #[must_use]
    pub const fn new(target_context: Internment<'source, 'context, S, L::Context>) -> Self {
        Self(target_context)
    }
}

impl<'source, 'context, S, L> TokenStream for LexStream<'source, 'context, S, L>
where
    S: Source<'source> + SourceDissect<'source> + SourceIter<'source> + ?Sized,
    L: Lex<'source, S>,
{
    type Token = L;
    type Error = L::Error;

    /// Consume one lexical item from the underlying source.
    ///
    /// Source exhaustion returns `None` without invoking the lexer. If lexing
    /// fails, cursor or context changes made by the lexer are not restored.
    #[inline]
    fn next(&mut self) -> Result<Option<Self::Token>, Self::Error> {
        let Self(state) = self;

        let (text, context) = state.parts_mut();

        if text.is_eof() {
            Ok(None)
        } else {
            L::lex_with_context(text, context).map(Some)
        }
    }
}

impl<'source, 'context, S, L> fmt::Debug for LexStream<'source, 'context, S, L>
where
    S: Source<'source> + SourceDissect<'source> + SourceIter<'source> + ?Sized,
    S::Component: fmt::Debug,
    L: Lex<'source, S>,
    L::Context: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self(target_value) = self;

        f.debug_tuple("LexStream").field(target_value).finish()
    }
}

#[cfg(test)]
mod tests {
    //! Tests module for the [`stream`] module.
    //!
    //! [`stream`]: super

    use aldebaran_grammar::stream::TokenStream;

    use crate::cursor::{Cursor, consume::Lex, internment::Internment};

    use super::LexStream;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    // NOTE(invariant): None.
    struct Counted(u8);

    impl<'source> Lex<'source, [u8]> for Counted {
        type Input<'input>
            = u8
        where
            'source: 'input;

        type Context = usize;

        type Error = ();

        #[inline]
        fn pass<'input>(text: &'input mut Cursor<'source, [u8]>) -> Result<Self::Input<'input>, Self::Error>
        where
            'source: 'input,
        {
            text.any().ok_or(())
        }

        #[inline]
        fn build_with_context<'input>(byte: Self::Input<'input>, count: &mut Self::Context) -> Result<Self, Self::Error>
        where
            'source: 'input,
        {
            *count = count.saturating_add(1);

            Ok(Self(byte))
        }
    }

    #[test]
    fn stream_reuses_one_context_and_stops_at_source_exhaustion() {
        let source = b"ab".as_slice();
        let mut count = 0_usize;

        {
            let internment = Internment::new(source, &mut count);
            let mut stream = LexStream::<[u8], Counted>::new(internment);

            assert_eq!(TokenStream::next(&mut stream), Ok(Some(Counted(b'a'))));
            assert_eq!(TokenStream::next(&mut stream), Ok(Some(Counted(b'b'))));
            assert_eq!(TokenStream::next(&mut stream), Ok(None));
        }

        assert_eq!(count, 2);
    }

    #[test]
    fn empty_source_never_invokes_the_lexer() {
        let mut count = 0_usize;

        {
            let source = b"".as_slice();
            let internment = Internment::new(source, &mut count);
            let mut stream = LexStream::<[u8], Counted>::new(internment);

            for _ in 0..3 {
                assert_eq!(stream.next(), Ok(None));
            }
        }

        assert_eq!(count, 0);
    }
}
