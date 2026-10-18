//! Human-readable and symbolic assertion formatters.
//!
//! Both parenthesize operators weaker than their surrounding context and preserve
//! custom segment text verbatim. Equal binding strengths require no parentheses.

use core::fmt;

use aldebaran_print::prelude::Print;

use super::{
    BinaryConnective, BuiltinBinaryConnective, BuiltinUnaryConnective, Connective, FormatWith, Formatter, Precedence, Segment,
    UnaryConnective,
};

/// Word-based notation used when an adapter's formatter is unspecified.
pub type DefaultFormatter = Human;

/// Word-based assertion notation using `and`, `or`, `xor`, and `not`.
/// Parentheses preserve grouping when an operator binds less tightly than its context.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Human;

impl Formatter for Human {
    #[inline]
    fn enter<W>(writer: &mut W, target_segment: Segment, precedence: Precedence) -> fmt::Result
    where
        W: fmt::Write,
    {
        match target_segment {
            Segment::Start(target_connective) => {
                let target_precedence = Connective::precedence(target_connective);

                if precedence > target_precedence {
                    write!(writer, "(")?;
                }

                match target_connective {
                    Connective::Unary(UnaryConnective::Builtin(target_connective)) => match target_connective {
                        BuiltinUnaryConnective::Not => write!(writer, "not "),
                    },
                    Connective::Unary(UnaryConnective::Any {
                        target_repr: Some(target_content),
                        ..
                    }) => {
                        write!(writer, "{target_content}")
                    }
                    Connective::Binary(BinaryConnective::Any {
                        target_repr: Some(target_content),
                        ..
                    }) => {
                        write!(writer, "{target_content}")
                    }
                    Connective::Binary(BinaryConnective::Builtin(_)) => Ok(()),
                    Connective::Unary(UnaryConnective::Any { target_repr: None, .. })
                    | Connective::Binary(BinaryConnective::Any { target_repr: None, .. }) => Ok(()),
                }
            }
            Segment::Midway(BinaryConnective::Builtin(target_connective)) => match target_connective {
                BuiltinBinaryConnective::And => write!(writer, " and "),
                BuiltinBinaryConnective::Or => write!(writer, " or "),
                BuiltinBinaryConnective::Xor => write!(writer, " xor "),
            },
            Segment::Midway(BinaryConnective::Any {
                target_repr: Some(target_content),
                ..
            }) => write!(writer, "{target_content}"),
            Segment::Midway(BinaryConnective::Any { target_repr: None, .. }) => Ok(()),
            Segment::End(target_connective) => {
                let target_precedence = Connective::precedence(target_connective);

                if precedence > target_precedence {
                    write!(writer, ")")?;
                }

                match target_connective {
                    Connective::Unary(UnaryConnective::Any {
                        target_repr: Some(target_content),
                        ..
                    }) => {
                        write!(writer, "{target_content}")
                    }
                    Connective::Binary(BinaryConnective::Any {
                        target_repr: Some(target_content),
                        ..
                    }) => {
                        write!(writer, "{target_content}")
                    }
                    Connective::Unary(UnaryConnective::Builtin(_)) | Connective::Binary(BinaryConnective::Builtin(_)) => Ok(()),
                    Connective::Unary(UnaryConnective::Any { target_repr: None, .. })
                    | Connective::Binary(BinaryConnective::Any { target_repr: None, .. }) => Ok(()),
                }
            }
        }
    }
}

impl<T> FormatWith<Human> for T
where
    T: Print,
    T::Context: Default,
{
    #[inline]
    fn output_with<W>(&self, writer: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        T::print(self, writer)
    }
}

/// Assertion notation using `∧`, `∨`, `⊻`, and `¬`.
/// It groups operators by precedence in the same way as [`Human`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Symbolic;

impl Formatter for Symbolic {
    #[inline]
    fn enter<W>(writer: &mut W, target_segment: Segment, precedence: Precedence) -> fmt::Result
    where
        W: fmt::Write,
    {
        match target_segment {
            Segment::Start(target_connective) => {
                let target_precedence = Connective::precedence(target_connective);

                if precedence > target_precedence {
                    write!(writer, "(")?;
                }

                match target_connective {
                    Connective::Unary(UnaryConnective::Builtin(target_connective)) => match target_connective {
                        BuiltinUnaryConnective::Not => write!(writer, "¬"),
                    },
                    Connective::Unary(UnaryConnective::Any {
                        target_repr: Some(target_content),
                        ..
                    }) => {
                        write!(writer, "{target_content}")
                    }
                    Connective::Binary(BinaryConnective::Any {
                        target_repr: Some(target_content),
                        ..
                    }) => {
                        write!(writer, "{target_content}")
                    }
                    Connective::Binary(BinaryConnective::Builtin(_)) => Ok(()),
                    Connective::Unary(UnaryConnective::Any { target_repr: None, .. })
                    | Connective::Binary(BinaryConnective::Any { target_repr: None, .. }) => Ok(()),
                }
            }
            Segment::Midway(BinaryConnective::Builtin(target_connective)) => match target_connective {
                BuiltinBinaryConnective::And => write!(writer, " ∧ "),
                BuiltinBinaryConnective::Or => write!(writer, " ∨ "),
                BuiltinBinaryConnective::Xor => write!(writer, " ⊻ "),
            },
            Segment::Midway(BinaryConnective::Any {
                target_repr: Some(target_content),
                ..
            }) => write!(writer, "{target_content}"),
            Segment::Midway(BinaryConnective::Any { target_repr: None, .. }) => Ok(()),
            Segment::End(target_connective) => {
                let target_precedence = Connective::precedence(target_connective);

                if precedence > target_precedence {
                    write!(writer, ")")?;
                }

                match target_connective {
                    Connective::Unary(UnaryConnective::Any {
                        target_repr: Some(target_content),
                        ..
                    }) => {
                        write!(writer, "{target_content}")
                    }
                    Connective::Binary(BinaryConnective::Any {
                        target_repr: Some(target_content),
                        ..
                    }) => {
                        write!(writer, "{target_content}")
                    }
                    Connective::Unary(UnaryConnective::Builtin(_)) | Connective::Binary(BinaryConnective::Builtin(_)) => Ok(()),
                    Connective::Unary(UnaryConnective::Any { target_repr: None, .. })
                    | Connective::Binary(BinaryConnective::Any { target_repr: None, .. }) => Ok(()),
                }
            }
        }
    }
}

impl<T> FormatWith<Symbolic> for T
where
    T: Print,
    T::Context: Default,
{
    #[inline]
    fn output_with<W>(&self, writer: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        T::print(self, writer)
    }
}
