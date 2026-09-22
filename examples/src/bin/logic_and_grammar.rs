//! Composable assertions and token stream primitives.

use aldebaran_grammar::prelude::{Lookahead, Slice, TokenStream};
use aldebaran_logic::prelude::{Assert, Choose, DefaultFormatter, OneOf};
use aldebaran_text::prelude::{AsciiAlphabetic, AsciiAlphanumeric, AsciiDigit, AsciiHexadecimal};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Assert, Choose)]
#[assert(disjunctive)]
struct IdentifierStart(AsciiAlphabetic, char);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Assert, Choose)]
#[assert(disjunctive)]
struct IdentifierContinue(AsciiAlphanumeric, char);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Assert, Choose)]
#[assert(conjunctive)]
struct HexLetter(AsciiAlphabetic, AsciiHexadecimal);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Assert, Choose)]
#[assert(use IdentifierStart(AsciiAlphabetic, '_') type IdentifierStart)]
struct NameStart;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Assert, Choose)]
enum SyntaxClass {
    #[assert(conjunctive)]
    HexLetter(AsciiAlphabetic, AsciiHexadecimal),

    #[assert(use IdentifierStart(AsciiAlphabetic, '_') type IdentifierStart)]
    IdentifierStart,

    #[assert(use AsciiDigit type AsciiDigit)]
    DecimalDigit,

    #[assert(disjunctive)]
    Delimiter(char, char),
}

fn main() {
    let identifier_start = IdentifierStart(AsciiAlphabetic, '_');
    let identifier_continue = IdentifierContinue(AsciiAlphanumeric, '_');
    let hex_letter = HexLetter(AsciiAlphabetic, AsciiHexadecimal);

    assert!(identifier_start.assert('_'));
    assert!(!identifier_start.assert('7'));
    assert!(identifier_continue.assert('7'));
    assert!(hex_letter.assert('f'));
    assert!(!hex_letter.assert('9'));
    assert_eq!(NameStart.choose('_'), Some(NameStart));

    let classes = OneOf::these([
        SyntaxClass::HexLetter(AsciiAlphabetic, AsciiHexadecimal),
        SyntaxClass::IdentifierStart,
        SyntaxClass::DecimalDigit,
        SyntaxClass::Delimiter('(', ')'),
    ]);

    let classified = [
        classes.choose('f').expect("hexadecimal letter"),
        classes.choose('_').expect("identifier start"),
        classes.choose('7').expect("decimal digit"),
        classes.choose(')').expect("delimiter"),
    ];

    assert_eq!(
        classified,
        [
            SyntaxClass::HexLetter(AsciiAlphabetic, AsciiHexadecimal),
            SyntaxClass::IdentifierStart,
            SyntaxClass::DecimalDigit,
            SyntaxClass::Delimiter('(', ')'),
        ]
    );

    let mut stream = Slice::new(&classified);

    assert_eq!(stream.lookahead::<0>().expect("slice streams are infallible"), Some(classified[0]));
    assert_eq!(stream.lookahead::<2>().expect("slice streams are infallible"), Some(classified[2]));
    assert_eq!(stream.next().expect("slice streams are infallible"), Some(classified[0]));
    assert_eq!(stream.next().expect("slice streams are infallible"), Some(classified[1]));

    let mut rendered = String::new();
    Assert::<char>::output::<_, DefaultFormatter>(&identifier_start, &mut rendered).expect("formatting into a string is infallible");

    println!("identifier-start assertion {rendered}");
    println!("classified syntax {classified:?}");
}
