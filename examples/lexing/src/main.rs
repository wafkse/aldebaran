//! Source traversal, lexical recognition, interning, and token-stream lookahead.

use aldebaran_grammar::prelude::{Lookahead, Skipping, Slice, TokenStream};
use aldebaran_logic::prelude::{Assert, Choose, OneOf};
use aldebaran_source::prelude::{Source, SourceDissect, SourceIter};
use aldebaran_text::prelude::{
    AsciiAlphabetic, AsciiAlphanumeric, AsciiDigit, AsciiWhitespace, Internment, Lex, LexStream, OwnedStorage, StorageId, Text,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Assert, Choose)]
#[assert(disjunctive)]
/// Accept an ASCII letter or the configured identifier prefix character.
struct IdentifierStart(AsciiAlphabetic, char);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Assert, Choose)]
#[assert(disjunctive)]
/// Accept an ASCII letter, digit, or the configured continuation character.
struct IdentifierContinue(AsciiAlphanumeric, char);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Assert, Choose)]
/// Recognize the first component of each supported lexical category.
enum LexemeHead {
    /// The first character of an identifier.
    #[assert(use IdentifierStart(AsciiAlphabetic, '_') type IdentifierStart)]
    Identifier,

    /// The first digit of an unsigned integer.
    #[assert(use AsciiDigit type AsciiDigit)]
    Integer,

    /// Whitespace omitted from the parser stream.
    #[assert(use AsciiWhitespace type AsciiWhitespace)]
    Trivia,

    /// Any configured punctuation character.
    #[assert(disjunctive)]
    Punctuation(char, char, char, char, char),
}

/// Ordered recognition policies for the example's lexical categories.
const LEXEME_HEADS: OneOf<LexemeHead, 4> = OneOf::these([
    LexemeHead::Identifier,
    LexemeHead::Integer,
    LexemeHead::Trivia,
    LexemeHead::Punctuation('=', '+', '*', '(', ')'),
]);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Borrowed lexical spelling passed to token construction.
enum LexemeInput<'source> {
    /// Identifier spelling awaiting internment.
    Identifier(&'source str),
    /// Decimal digits awaiting numeric conversion.
    Integer(&'source str),
    /// Consumed whitespace with no retained spelling.
    Trivia,
    /// A recognized punctuation character.
    Punctuation(char),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Semantic tokens emitted after lexical conversion and internment.
enum Token {
    /// The stable identity of an interned identifier.
    Identifier(StorageId),
    /// A representable unsigned integer value.
    Integer(u64),
    /// Whitespace retained until the skipping adapter removes it.
    Trivia,
    /// A recognized punctuation character.
    Punctuation(char),
}

impl Token {
    /// Identify whitespace tokens skipped before parsing.
    const fn is_trivia(&self) -> bool {
        matches!(self, Self::Trivia)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Failures from lexical recognition, conversion, or identifier storage.
enum LexError {
    /// The next component does not begin a supported lexeme.
    UnexpectedCharacter {
        /// The unrecognized source character.
        found: char,

        /// Its absolute byte offset in the source.
        index: usize,
    },
    /// Source exhaustion before a required component.
    UnexpectedEnd {
        /// The source boundary where a component was required.
        index: usize,
    },
    /// Decimal digits cannot be represented by the token's integer type.
    IntegerOverflow,
    /// Identifier storage cannot assign another stable identity.
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
            LexemeInput::Integer(source) => source
                .parse::<u64>()
                .map(Self::Integer)
                .map_err(|_parse_error| LexError::IntegerOverflow),
            LexemeInput::Trivia => Ok(Self::Trivia),
            LexemeInput::Punctuation(punctuation) => Ok(Self::Punctuation(punctuation)),
        }
    }
}

fn main() {
    let source = "sum_2 = alpha42 + 17*(sum_2)\n";
    let footprint = Source::footprint(source).expect("source is nonempty");
    let reconstructed: String = SourceIter::iter(source).collect();

    assert_eq!(footprint.range(), 0..source.len(), "the footprint covers the entire source");
    assert_eq!(reconstructed, source, "source iteration preserves every component");

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

    assert_eq!(tokens.len(), 9, "trivia filtering preserves the nine semantic tokens");

    let first_identifier = match tokens[0] {
        Token::Identifier(identifier) => Some(identifier),
        _ => None,
    }
    .expect("first token should be an identifier");
    let repeated_identifier = match tokens[7] {
        Token::Identifier(identifier) => Some(identifier),
        _ => None,
    }
    .expect("eighth token should be the repeated identifier");

    assert_eq!(first_identifier, repeated_identifier, "equal spellings share an interned identity");
    assert_eq!(
        storage.try_resolve(first_identifier),
        Some("sum_2"),
        "internment retains the identifier spelling"
    );
    assert_eq!(tokens[4], Token::Integer(17), "decimal digits produce their integer value");
    assert_eq!(tokens[5], Token::Punctuation('*'), "punctuation is preserved");

    let mut parser_stream = Slice::new(tokens.as_slice());
    let first = parser_stream.lookahead::<0>().expect("slice streams are infallible");
    let third = parser_stream.lookahead::<2>().expect("slice streams are infallible");

    assert_eq!(first, Some(tokens[0]), "immediate lookahead sees the first token");
    assert_eq!(third, Some(tokens[2]), "indexed lookahead sees the requested token");
    assert_eq!(
        parser_stream.next().expect("slice streams are infallible"),
        Some(tokens[0]),
        "lookahead does not consume tokens"
    );

    println!("source footprint {footprint:?}");
    println!("visible tokens {tokens:?}");
    println!("repeated identifier shares {first_identifier}");
    println!("parser lookahead {first:?}, then {third:?}");
}
