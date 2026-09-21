//! Structured formatting of assertions and nested logical connectives.
//!
//! Formatter implementations receive connective boundaries plus precedence
//! rather than concrete assertion types. This separates expression structure
//! from presentation and allows human-readable or symbolic output to share logic.

pub mod common;

use core::{any::TypeId, cmp, num::NonZero};

use core::fmt;

use aldebaran_print::prelude::Print;

/// A trait for one-shot formatters.
///
/// One-shot formatters are those who dictate only by two parameters: the
/// current [`Precedence`], and the inherent [`Segment`].
///
///
/// We achieve this by operating on the *positional context*  rather than the
/// specific parts of the assertion. This allows us to abstract over the
/// specifics of each assertion type, and focus on the general structure of the
/// assertion.
/// In the end, this results in pretty-printed assertions that are easy to
/// read and understand, and that have clear precedence rules.
///
/// [`Assert`]: crate::assert::Assert
pub trait Formatter {
    /// Perform a format operation on the specified writer.
    fn enter<W>(writer: &mut W, target_segment: Segment, target_precedence: Precedence) -> fmt::Result
    where
        W: fmt::Write;

    /// Enter a [`Segment::Start`] in this formatter.
    #[inline]
    fn start<W>(writer: &mut W, target_connective: Connective, target_precedence: Precedence) -> fmt::Result
    where
        W: fmt::Write,
    {
        Self::enter(writer, Segment::Start(target_connective), target_precedence)
    }

    /// Enter a [`Segment::Midway`] in this formatter.
    #[inline]
    fn middle<W>(writer: &mut W, target_connective: BinaryConnective, target_precedence: Precedence) -> fmt::Result
    where
        W: fmt::Write,
    {
        Self::enter(writer, Segment::Midway(target_connective), target_precedence)
    }

    /// Enter a [`Segment::End`] in this formatter.
    #[inline]
    fn end<W>(writer: &mut W, target_connective: Connective, target_precedence: Precedence) -> fmt::Result
    where
        W: fmt::Write,
    {
        Self::enter(writer, Segment::End(target_connective), target_precedence)
    }
}

/// A trait for those types that can be formatted regardless of the underlying
/// formatter.
pub trait Format {
    /// Pretty-print this type to the specified [`fmt::Write`] `W`
    fn output<W>(&self, writer: &mut W) -> fmt::Result
    where
        W: fmt::Write;
}

impl<T> Format for T
where
    T: Print,
    T::Context: Default,
{
    #[inline]
    fn output<W>(&self, writer: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        T::print(self, writer)
    }
}

/// A trait for types that are specialized to a target [`Formatter`] `F`.
pub trait FormatWith<F>
where
    F: Formatter,
{
    /// Pretty-print this type to the specified [`fmt::Write`] `W`
    fn output_with<W>(&self, writer: &mut W) -> fmt::Result
    where
        W: fmt::Write;
}

/// A format segment.
///
/// This is an enumeration of the relevant portions of a formatted expression,
/// where a [`Formatter`] can have an effect,
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Segment {
    /// The start of a connective.
    ///
    /// This represents the "place" where an arbitrary operator is situated.
    ///
    /// See [`Connective`] for more information.
    Start(Connective),

    /// Somewhere in the middle of a connective.
    ///
    /// This represents the "place" where a binary operator is situated.
    ///
    /// This can only be a binary operator, as unary operators are never infix.
    ///
    /// See [`BinaryConnective`] for more information.
    Midway(BinaryConnective),

    /// The end of a connective.
    ///
    /// This represents the "place" where an arbitrary operator is situated.
    ///
    /// This can be either a unary operator or a binary operator.
    ///
    /// See [`Connective`] for more information.
    End(Connective),
}

impl Segment {
    /// Determine the precedence of the segment.
    #[inline]
    pub const fn precedence(target_segment: Self) -> Precedence {
        match target_segment {
            Self::Start(connective) => Connective::precedence(connective),
            Self::Midway(binary) => BinaryConnective::precedence(binary),
            Self::End(connective) => Connective::precedence(connective),
        }
    }
}

/// A logical connective.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Connective {
    /// An 1-ary connective.
    Unary(UnaryConnective),

    /// A 2-ary connective.
    Binary(BinaryConnective),
}

impl Connective {
    /// A constant that is the built-in `and` connective.
    pub const AND: Self = Self::Binary(BinaryConnective::Builtin(BuiltinBinaryConnective::And));

    /// A constant that is the built-in `or` connective.
    pub const OR: Self = Self::Binary(BinaryConnective::Builtin(BuiltinBinaryConnective::Or));

    /// A constant that is the built-in `xor` connective.
    pub const XOR: Self = Self::Binary(BinaryConnective::Builtin(BuiltinBinaryConnective::Xor));

    /// A constant that is the built-in `not` connective.
    pub const NOT: Self = Self::Unary(UnaryConnective::Builtin(BuiltinUnaryConnective::Not));
}

impl Connective {
    /// Determine the precedence of this connective.
    #[inline]
    pub const fn precedence(target_connective: Self) -> Precedence {
        match target_connective {
            Self::Unary(unary) => UnaryConnective::precedence(unary),
            Self::Binary(binary) => BinaryConnective::precedence(binary),
        }
    }
}

/// A 2-ary connective.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BinaryConnective {
    /// A built-in 2-ary connective.
    Builtin(BuiltinBinaryConnective),

    /// An arbitrary 2-adic connective.
    Any {
        /// The potentially-missing representation of the connective at hand.
        ///
        /// Note that this is not global to this connective, but is specific to
        /// some formatter [`Segment`].
        target_repr: Option<&'static str>,

        /// The type id of the target connective.
        target_id: TypeId,

        /// The precedence of the connective.
        target_precedence: Precedence,
    },
}

impl BinaryConnective {
    /// A constant that is the built-in `and` 2-ary connective.
    pub const AND: Self = Self::Builtin(BuiltinBinaryConnective::And);

    /// A constant that is the built-in `or` 2-ary connective.
    pub const OR: Self = Self::Builtin(BuiltinBinaryConnective::Or);

    /// A constant that is the built-in `xor` 2-ary connective.
    pub const XOR: Self = Self::Builtin(BuiltinBinaryConnective::Xor);
}

impl BinaryConnective {
    /// Determine the precedence of the target binary connective.
    #[inline]
    pub const fn precedence(target_connective: Self) -> Precedence {
        match target_connective {
            Self::Builtin(builtin) => BuiltinBinaryConnective::precedence(builtin),
            Self::Any { target_precedence, .. } => target_precedence,
        }
    }
}

/// A 2-ary connective that is built-in to the library.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BuiltinBinaryConnective {
    /// The `and` 2-ary connective.
    And,

    /// The `or` 2-ary connective.
    Or,

    /// The `xor` 2-ary connective.
    Xor,
}

impl BuiltinBinaryConnective {
    /// Determine the [`Precedence`] of the target built-in binary connective.
    #[inline]
    pub const fn precedence(target_connective: Self) -> Precedence {
        match target_connective {
            Self::And => Precedence::AND,
            Self::Or => Precedence::OR,
            Self::Xor => Precedence::XOR,
        }
    }
}

/// An 1-ary connective.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UnaryConnective {
    /// A built-in 1-ary connective.
    Builtin(BuiltinUnaryConnective),

    /// An arbitrary n-adic connective.
    Any {
        /// The potentially-missing representation of the connective at hand.
        ///
        /// Note that this is not global to this connective, but is specific to
        target_repr: Option<&'static str>,

        /// The type id of the target connective.
        target_id: TypeId,

        /// The inherent precedence of the connective.
        target_precedence: Precedence,
    },
}

impl UnaryConnective {
    /// A constant that is the built-in `not` 1-ary connective.
    pub const NOT: Self = Self::Builtin(BuiltinUnaryConnective::Not);
}

impl UnaryConnective {
    /// Determine the precedence of the target unary connective.
    #[inline]
    pub const fn precedence(target_connective: Self) -> Precedence {
        match target_connective {
            Self::Builtin(builtin) => BuiltinUnaryConnective::precedence(builtin),
            Self::Any { target_precedence, .. } => target_precedence,
        }
    }
}

/// An 1-ary connective that is built-in to the library.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BuiltinUnaryConnective {
    /// The `not` 1-ary connective.
    Not,
}

impl BuiltinUnaryConnective {
    /// Determine the [`Precedence`] of the target built-in unary connective.
    #[inline]
    pub const fn precedence(target_connective: Self) -> Precedence {
        match target_connective {
            Self::Not => Precedence::NOT,
        }
    }
}

/// A potentially-none precedence level to rule the appearance of parentheses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Precedence {
    /// A non-existent precedence level.
    #[default]
    None,

    /// A distinct precedence level to rule the appearance of parentheses.
    Level(NonZero<usize>),
}

impl Precedence {
    /// The minimum precedence level that is exactly above [`Precedence::None`].
    pub const MIN: Self = Self::Level(NonZero::<usize>::MIN);

    /// The precedence level of a logical disjunction.
    pub const OR: Self = Self::MIN;

    /// The precedence level of a logical exclusive disjunction.
    pub const XOR: Self = Self::increase(Self::OR);

    /// The precedence level of a logical conjunction.
    pub const AND: Self = Self::increase(Self::XOR);

    /// The precedence level of a logical negation.
    pub const NOT: Self = Self::increase(Self::AND);
}

impl Precedence {
    /// Determine the precedence level associated to a target [`usize`] value.
    #[inline]
    pub const fn integer(target_value: usize) -> Option<Self> {
        match NonZero::new(target_value) {
            Some(target_level) => Some(Self::Level(target_level)),
            None => None,
        }
    }

    /// Determine the precedence level associated to a target [`Precedence`]
    /// value.
    #[inline]
    pub const fn value(target_precedence: Self) -> usize {
        match target_precedence {
            Self::None => 0,
            Self::Level(target_level) => target_level.get(),
        }
    }

    /// Determine the immediate predecessor of the target precedence level.
    ///
    /// If the target precedence level is
    /// `Precedence::Level(NonZero::<usize>::MAX)`, the result will be
    /// saturated.
    #[inline]
    pub const fn increase(target_precedence: Self) -> Self {
        match target_precedence {
            Self::None => Self::MIN,
            Self::Level(level) => Self::Level(level.saturating_add(1)),
        }
    }
}

impl PartialOrd for Precedence {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(Self::cmp(self, other))
    }
}

impl Ord for Precedence {
    #[inline]
    fn cmp(&self, &other: &Self) -> cmp::Ordering {
        let (lhs, ref rhs) = (Self::value(*self), Self::value(other));

        lhs.cmp(rhs)
    }
}
