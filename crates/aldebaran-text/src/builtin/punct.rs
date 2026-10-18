//! Named alternatives for ASCII opening and closing delimiters. Each enum value
//! matches its selected side for byte or character input and imposes no balancing rule.

use aldebaran_logic::prelude::Assert;

use super::Ascii;

/// Selects a parenthesis side as an ASCII equality predicate for either source kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Assert)]
pub enum Parenthesis {
    /// A left parenthesis `(`.
    #[assert(use Ascii::LeftParenthesis type Ascii)]
    Left,

    /// A right parenthesis `)`.
    #[assert(use Ascii::RightParenthesis type Ascii)]
    Right,
}

/// Selects a square-bracket side as an ASCII equality predicate for either source kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Assert)]
pub enum Bracket {
    /// A left bracket `[`.
    #[assert(use Ascii::LeftBracket type Ascii)]
    Left,

    /// A right bracket `]`.
    #[assert(use Ascii::RightBracket type Ascii)]
    Right,
}

/// Selects a brace side as an ASCII equality predicate for either source kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Assert)]
pub enum Brace {
    /// A left brace `{`.
    #[assert(use Ascii::LeftBrace type Ascii)]
    Left,

    /// A right brace `}`.
    #[assert(use Ascii::RightBrace type Ascii)]
    Right,
}

/// Selects an angle-bracket side as an ASCII equality predicate for either source kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Assert)]
pub enum AngleBracket {
    /// A left angle bracket `<`.
    #[assert(use Ascii::LessThan type Ascii)]
    Left,

    /// A right angle bracket `>`.
    #[assert(use Ascii::GreaterThan type Ascii)]
    Right,
}
