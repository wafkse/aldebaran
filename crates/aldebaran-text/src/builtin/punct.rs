//! Semantic punctuation assertions built from ASCII predicates.
//!
//! See [`Parenthesis`], [`Bracket`], [`Brace`], and [`AngleBracket`] for further information.
//!
//! Each variant matches one ASCII delimiter for byte or character input, without
//! checking whether opening and closing delimiters balance.

use aldebaran_logic::prelude::Assert;

use super::Ascii;

/// A parenthesis character, regardless of its direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Assert)]
#[aldebaran(crate)]
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
#[aldebaran(crate)]
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
#[aldebaran(crate)]
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
#[aldebaran(crate)]
pub enum AngleBracket {
    /// A left angle bracket `<`.
    #[assert(use Ascii::LessThan type Ascii)]
    Left,

    /// A right angle bracket `>`.
    #[assert(use Ascii::GreaterThan type Ascii)]
    Right,
}

#[cfg(test)]
mod tests {
    use super::{AngleBracket, Brace, Bracket, Parenthesis};

    use aldebaran_logic::prelude::Assert;

    macro_rules! assert_delimiter {
        ($assertion:expr, $target_char:literal, $other_char:literal, $target_byte:literal, $other_byte:literal) => {{
            let assertion = $assertion;
            let target_char = $target_char;
            let other_char = $other_char;
            let target_byte = $target_byte;
            let other_byte = $other_byte;

            assert!(Assert::<char>::assert(&assertion, target_char));
            assert!(Assert::<&char>::assert(&assertion, &target_char));
            assert!(!Assert::<char>::assert(&assertion, other_char));
            assert!(!Assert::<&char>::assert(&assertion, &other_char));

            assert!(Assert::<u8>::assert(&assertion, target_byte));
            assert!(Assert::<&u8>::assert(&assertion, &target_byte));
            assert!(!Assert::<u8>::assert(&assertion, other_byte));
            assert!(!Assert::<&u8>::assert(&assertion, &other_byte));
        }};
    }

    #[test]
    fn delimiter_assertions_match_each_direction_for_value_and_reference_inputs() {
        assert_delimiter!(Parenthesis::Left, '(', ')', b'(', b')');
        assert_delimiter!(Parenthesis::Right, ')', '(', b')', b'(');
        assert_delimiter!(Bracket::Left, '[', ']', b'[', b']');
        assert_delimiter!(Bracket::Right, ']', '[', b']', b'[');
        assert_delimiter!(Brace::Left, '{', '}', b'{', b'}');
        assert_delimiter!(Brace::Right, '}', '{', b'}', b'{');
        assert_delimiter!(AngleBracket::Left, '<', '>', b'<', b'>');
        assert_delimiter!(AngleBracket::Right, '>', '<', b'>', b'<');
    }
}
