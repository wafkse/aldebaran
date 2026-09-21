//! Incremental token streams backed directly by lexical source consumption.
//!
//! [`LexStream`] repeatedly invokes one lexical type against a persistent cursor
//! and mutable context. Source exhaustion maps directly to `Ok(None)` and never
//! fabricates an EOF token or invokes the lexical implementation after exhaustion.

use aldebaran_grammar::stream::TokenStream;
use aldebaran_source::prelude::{Source, SourceDissect, SourceIter};

use super::{consume::Lex, internment::Internment};

/// A lexical token stream over one source cursor and persistent mutable context.
///
/// Each successful stream step recognizes and constructs exactly one lexical
/// value through `L`. The wrapped [`Internment`] guarantees that every step sees
/// the same construction context and advances the same source cursor.
#[derive(Debug)]
pub struct LexStream<'source, 'context, S, L>(pub Internment<'source, 'context, S, L::Context>)
where
    S: Source<'source> + SourceDissect<'source> + SourceIter<'source> + ?Sized,
    L: Lex<'source, S>;

// NOTE(invariant): `state` owns the only source cursor and one persistent mutable
// context borrow. Every successful stream step lexes exactly one `L` with that
// context. Exhaustion is represented only by `Ok(None)` and never invokes `Lex`.
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
    S: Source<'source> + SourceDissect<'source> + SourceIter<'source> + ?Sized + 'source,
    L: Lex<'source, S>,
{
    type Token = L;
    type Error = L::Error;

    /// Consume one lexical item from the underlying source.
    #[inline]
    fn next(&mut self) -> Result<Option<Self::Token>, Self::Error> {
        let Self(state) = self;

        let (text, context) = state.parts_mut();

        if text.is_eof() {
            Ok(None)
        } else {
            L::lex_with_ctx(text, context).map(Some)
        }
    }
}

#[cfg(test)]
mod tests {
    use aldebaran_grammar::stream::TokenStream;

    use crate::text::{Text, consume::Lex, internment::Internment};

    use super::LexStream;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Counted(u8);

    impl<'source> Lex<'source, [u8]> for Counted {
        type Input<'input>
            = u8
        where
            'source: 'input;

        type Context = usize;

        type Error = ();

        #[inline]
        fn pass<'input>(text: &'input mut Text<'source, [u8]>) -> Result<Self::Input<'input>, Self::Error>
        where
            'source: 'input,
        {
            text.any().ok_or(())
        }

        #[inline]
        fn build_with_ctx<'input>(byte: Self::Input<'input>, count: &mut Self::Context) -> Result<Self, Self::Error>
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
        let mut count = 0usize;

        {
            let internment = Internment::new(source, &mut count);
            let mut stream = LexStream::<[u8], Counted>::new(internment);

            assert_eq!(TokenStream::next(&mut stream), Ok(Some(Counted(b'a'))));
            assert_eq!(TokenStream::next(&mut stream), Ok(Some(Counted(b'b'))));
            assert_eq!(TokenStream::next(&mut stream), Ok(None));
        }

        assert_eq!(count, 2);
    }
}
