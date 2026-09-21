//! Common formatter policies for structured logical assertions.
//!
//! [`Human`] writes word-oriented operators while [`Symbolic`] uses mathematical
//! connective symbols. Both honor the same precedence model and insert grouping
//! only where the surrounding expression requires it.

use core::fmt;

use aldebaran_print::prelude::Print;

use super::{
    BinaryConnective, BuiltinBinaryConnective, BuiltinUnaryConnective, Connective, FormatWith, Formatter, Precedence, Segment,
    UnaryConnective,
};

/// The default [`Formatter`] to be used.
pub type DefaultFormatter = Human;

/// A human-readable printer for [`Assert`](crate::assert::Assert) specifications.
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

/// A symbolic printer for [`Assert`](crate::assert::Assert) specifications.
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
