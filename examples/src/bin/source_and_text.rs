//! Source traversal, lexical recognition, interning, and token streaming.

use aldebaran_grammar::prelude::{Skipping, TokenStream};
use aldebaran_logic::prelude::{Assert, Choose, OneOf};
use aldebaran_source::prelude::{Source, SourceDissect, SourceIter};
use aldebaran_text::prelude::{
    AsciiAlphabetic, AsciiAlphanumeric, AsciiDigit, AsciiWhitespace, Internment, Lex, LexStream, OwnedStorage, StorageId, Text,
};
use aldebaran_visualize::prelude::Visualize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Assert, Choose)]
#[assert(disjunctive)]
struct IdentifierStart(AsciiAlphabetic, char);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Assert, Choose)]
#[assert(disjunctive)]
struct IdentifierContinue(AsciiAlphanumeric, char);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Assert, Choose)]
enum LexemeHead {
    #[assert(use IdentifierStart(AsciiAlphabetic, '_') type IdentifierStart)]
    Identifier,

    #[assert(use AsciiDigit type AsciiDigit)]
    Integer,

    #[assert(use AsciiWhitespace type AsciiWhitespace)]
    Trivia,

    #[assert(disjunctive)]
    Punctuation(char, char, char, char, char),
}

const LEXEME_HEADS: OneOf<LexemeHead, 4> = OneOf::these([
    LexemeHead::Identifier,
    LexemeHead::Integer,
    LexemeHead::Trivia,
    LexemeHead::Punctuation('=', '+', '*', '(', ')'),
]);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LexemeInput<'source> {
    Identifier(&'source str),
    Integer(&'source str),
    Trivia,
    Punctuation(char),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Token {
    Identifier(StorageId),
    Integer(u64),
    Trivia,
    Punctuation(char),
}

impl Token {
    const fn is_trivia(&self) -> bool {
        matches!(self, Self::Trivia)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LexError {
    UnexpectedCharacter { found: char, index: usize },
    UnexpectedEnd { index: usize },
    IntegerOverflow,
    InternmentFull,
}

impl<'source> Lex<'source, str> for Token {
    type Input<'input>
        = LexemeInput<'source>
    where
        'source: 'input;

    type Context = OwnedStorage<str>;

    type Error = LexError;

    fn pass<'input>(text: &'input mut Text<'source, str>) -> Result<Self::Input<'input>, Self::Error>
    where
        'source: 'input,
    {
        let index = text.index();
        let found = text.peek().ok_or(LexError::UnexpectedEnd { index })?;
        let head = LEXEME_HEADS.choose(found).ok_or(LexError::UnexpectedCharacter { found, index })?;

        match head {
            LexemeHead::Identifier => {
                let (_, first_span) = text.any_spanned().ok_or(LexError::UnexpectedEnd { index })?;
                let rest_span = text.ignore_while_spanned(IdentifierContinue(AsciiAlphanumeric, '_'));
                let span = rest_span.map_or(first_span, |rest_span| first_span.superset(rest_span));
                let source = text.source().dissect(span);

                Ok(LexemeInput::Identifier(source))
            }
            LexemeHead::Integer => {
                let (_, first_span) = text.any_spanned().ok_or(LexError::UnexpectedEnd { index })?;
                let rest_span = text.ignore_while_spanned(AsciiDigit);
                let span = rest_span.map_or(first_span, |rest_span| first_span.superset(rest_span));
                let source = text.source().dissect(span);

                Ok(LexemeInput::Integer(source))
            }
            LexemeHead::Trivia => {
                text.ignore_while(AsciiWhitespace);

                Ok(LexemeInput::Trivia)
            }
            LexemeHead::Punctuation(..) => {
                let punctuation = text.any().ok_or(LexError::UnexpectedEnd { index })?;

                Ok(LexemeInput::Punctuation(punctuation))
            }
        }
    }

    fn build_with_ctx<'input>(input: Self::Input<'input>, context: &mut Self::Context) -> Result<Self, Self::Error>
    where
        'source: 'input,
    {
        match input {
            LexemeInput::Identifier(source) => context.try_store(source).map(Self::Identifier).ok_or(LexError::InternmentFull),
            LexemeInput::Integer(source) => source.parse::<u64>().map(Self::Integer).map_err(|_| LexError::IntegerOverflow),
            LexemeInput::Trivia => Ok(Self::Trivia),
            LexemeInput::Punctuation(punctuation) => Ok(Self::Punctuation(punctuation)),
        }
    }
}

fn main() {
    let source = "sum_2 = alpha42 + 17*(sum_2)\n";
    let footprint = Source::footprint(source).expect("source is nonempty");
    let reconstructed: String = SourceIter::iter(source).collect();

    assert_eq!(footprint.range(), 0..source.len());
    assert_eq!(reconstructed, source);

    let mut storage: OwnedStorage<str> = OwnedStorage::empty();
    let tokens = {
        let internment = Internment::new(source, &mut storage);
        let lexed = LexStream::<str, Token>::new(internment);
        let mut visible = Skipping::new(lexed, Token::is_trivia);
        let mut tokens = Vec::new();

        while let Some(token) = visible.next().expect("example source is lexically valid") {
            tokens.push(token);
        }

        tokens
    };

    assert_eq!(tokens.len(), 9);

    let first_identifier = match tokens[0] {
        Token::Identifier(identifier) => identifier,
        _ => panic!("first token should be an identifier"),
    };
    let repeated_identifier = match tokens[7] {
        Token::Identifier(identifier) => identifier,
        _ => panic!("eighth token should be the repeated identifier"),
    };

    assert_eq!(first_identifier, repeated_identifier);
    assert_eq!(storage.try_resolve(first_identifier), Some("sum_2"));
    assert_eq!(tokens[4], Token::Integer(17));
    assert_eq!(tokens[5], Token::Punctuation('*'));

    let bytes = [b'A', 0xff, b'Z'];
    assert_eq!(format!("{}", bytes.as_slice().visual()), "A�Z");

    println!("source footprint {footprint:?}");
    println!("visible tokens {tokens:?}");
    println!("repeated identifier shares {first_identifier}");
}
