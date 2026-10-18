//! Semantic punctuation assertions built from ASCII predicates.
//!
//! These small enums classify directional punctuation families such as brackets,
//! braces, parentheses, and angle brackets. They are useful when syntax accepts
//! either direction while still requiring a structured assertion value.

use aldebaran_logic::prelude::Assert;

use super::Ascii;

/// A parenthesis character, regardless of its direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Assert)]
pub enum Parenthesis {
    /// A left parenthesis `(`.
    #[assert(use Ascii::LeftParenthesis type Ascii)]
    Left,

    /// A right parenthesis `)`.
    #[assert(use Ascii::RightParenthesis type Ascii)]
    Right,
}

/// A bracket character, regardless of its direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Assert)]
pub enum Bracket {
    /// A left bracket `[`.
    #[assert(use Ascii::LeftBracket type Ascii)]
    Left,

    /// A right bracket `]`.
    #[assert(use Ascii::RightBracket type Ascii)]
    Right,
}

/// A brace character, regardless of its direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Assert)]
pub enum Brace {
    /// A left brace `{`.
    #[assert(use Ascii::LeftBrace type Ascii)]
    Left,

    /// A right brace `}`.
    #[assert(use Ascii::RightBrace type Ascii)]
    Right,
}

/// An angle bracket character, regardless of its direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Assert)]
pub enum AngleBracket {
    /// A left angle bracket `<`.
    #[assert(use Ascii::LessThan type Ascii)]
    Left,

    /// A right angle bracket `>`.
    #[assert(use Ascii::GreaterThan type Ascii)]
    Right,
}
