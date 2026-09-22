//! Integration coverage for assertion derives and logical combinators.

use core::fmt;

use aldebaran_logic::{
    fmt::{Formatter, Precedence},
    prelude::{Assert, Choose, Connective as _, OneOf, Select},
};

macro_rules! assertion {
    ($name:ident, $character:literal, $byte:literal) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        struct $name;

        impl Assert<char> for $name {
            fn assert(&self, input: char) -> bool {
                input == $character
            }

            fn output_with<W, F>(_: &Self, writer: &mut W, _: Precedence) -> fmt::Result
            where
                W: fmt::Write,
                F: Formatter,
            {
                writer.write_char($character)
            }
        }

        impl Assert<u8> for $name {
            fn assert(&self, input: u8) -> bool {
                input == $byte
            }

            fn output_with<W, F>(_: &Self, writer: &mut W, _: Precedence) -> fmt::Result
            where
                W: fmt::Write,
                F: Formatter,
            {
                writer.write_char($character)
            }
        }

        impl Choose<char> for $name {
            type Unit = Self;

            fn choose(&self, input: char) -> Option<Self::Unit> {
                self.assert(input).then_some(Self)
            }
        }

        impl Choose<u8> for $name {
            type Unit = Self;

            fn choose(&self, input: u8) -> Option<Self::Unit> {
                self.assert(input).then_some(Self)
            }
        }
    };
}

assertion!(Left, '(', b'(');
assertion!(Right, ')', b')');

#[derive(Assert)]
#[assert(use Left type Left)]
struct PredicateOnly;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Assert, Choose)]
enum Token {
    #[assert(use Left type Left)]
    Left,

    #[assert(use Right type Right)]
    Right,
}

#[test]
fn predicate_only_derive_does_not_require_clone() {
    assert!(PredicateOnly.assert('('));
    assert!(!PredicateOnly.assert(')'));
}

#[test]
fn derived_classifier_chooses_itself_for_character_and_byte_inputs() {
    assert_eq!(Token::Left.choose('('), Some(Token::Left));
    assert_eq!(Token::Right.choose(b')'), Some(Token::Right));
}

#[test]
fn one_of_returns_the_selected_derived_assertion() {
    let assertions = OneOf::these([Token::Left, Token::Right]);

    assert_eq!(assertions.choose(')'), Some(Token::Right));
    assert_eq!(assertions.choose(b'('), Some(Token::Left));
}

#[test]
fn heterogeneous_or_returns_the_selected_assertion_family() {
    let assertions = Left.or(Right);

    assert_eq!(assertions.choose('('), Some(Select::First(Left)));
    assert_eq!(assertions.choose(b')'), Some(Select::Second(Right)));
}
