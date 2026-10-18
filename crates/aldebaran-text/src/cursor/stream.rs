//! Lexical values exposed through the parser's [`TokenStream`] interface.
//! [`LexStream`] retains source progress and construction state between reads.

use aldebaran_grammar::stream::TokenStream;
use aldebaran_source::prelude::{Source, SourceDissect, SourceIter};

use super::{consume::Lex, internment::Internment};

/// Produces lexical tokens with source progress and construction context shared
/// across reads. Exhaustion returns `None` without invoking the lexer, while
/// failures retain any cursor and context effects the lexer leaves behind.
#[derive(Debug)]
pub struct LexStream<'source, 'context, S, L>(pub Internment<'source, 'context, S, L::Context>)
where
    S: Source<'source> + SourceDissect<'source> + SourceIter<'source> + ?Sized,
    L: Lex<'source, S>;

impl<'source, 'context, S, L> LexStream<'source, 'context, S, L>
where
    S: Source<'source> + SourceDissect<'source> + SourceIter<'source> + ?Sized,
    L: Lex<'source, S>,
{
    /// Continue lexical production from the supplied source position and shared context.
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

    /// Read the next lexical value, returning `None` at source exhaustion.
    ///
    /// A failed read does not restore cursor or context changes made by the lexer.
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

#[cfg(test)]
mod tests;
