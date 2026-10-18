//! Assertion descriptions with explicit operator identity and binding strength.
//! [`Formatter`] controls connective notation and grouping, while [`Format`]
//! supplies descriptions for values that need no connective context.

pub mod common;

use core::{any::TypeId, cmp, num::NonZero};

use core::fmt;

use aldebaran_print::prelude::Print;

/// A notation and grouping policy for assertion descriptions. Each [`Segment`]
/// identifies part of an operator, while the supplied [`Precedence`] describes
/// the surrounding expression whose grouping must be preserved.
pub trait Formatter {
    /// Render operator text and any grouping needed in the surrounding context.
    fn enter<W>(writer: &mut W, target_segment: Segment, target_precedence: Precedence) -> fmt::Result
    where
        W: fmt::Write;

    /// Establish grouping and any prefix text before the connective's operands.
    #[inline]
    fn start<W>(writer: &mut W, target_connective: Connective, target_precedence: Precedence) -> fmt::Result
    where
        W: fmt::Write,
    {
        Self::enter(writer, Segment::Start(target_connective), target_precedence)
    }

    /// Separate operands using the selected binary operator's notation.
    #[inline]
    fn middle<W>(writer: &mut W, target_connective: BinaryConnective, target_precedence: Precedence) -> fmt::Result
    where
        W: fmt::Write,
    {
        Self::enter(writer, Segment::Midway(target_connective), target_precedence)
    }

    /// Close grouping and any suffix text after the connective's operands.
    #[inline]
    fn end<W>(writer: &mut W, target_connective: Connective, target_precedence: Precedence) -> fmt::Result
    where
        W: fmt::Write,
    {
        Self::enter(writer, Segment::End(target_connective), target_precedence)
    }
}

/// A description that needs no surrounding operator context. For inputs implementing
/// [`Print`] with a default context, it also describes exact-equality assertions.
pub trait Format {
    /// Describe the value independently of any connective notation or precedence.
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

/// A description whose notation is selected by `F`. This permits different
/// representations of the same value without changing its assertion semantics.
pub trait FormatWith<F>
where
    F: Formatter,
{
    /// Describe the value under `F`'s notation policy.
    fn output_with<W>(&self, writer: &mut W) -> fmt::Result
    where
        W: fmt::Write;
}

/// A connective segment passed to a [`Formatter`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Segment {
    /// The opening boundary of a unary or binary connective.
    Start(Connective),

    /// A binary operator between its operands.
    ///
    /// Unary connectives have no midway segment.
    Midway(BinaryConnective),

    /// The closing boundary of a unary or binary connective.
    End(Connective),
}

impl Segment {
    /// Binding strength shared by every segment belonging to this operator.
    #[inline]
    pub const fn precedence(target_segment: Self) -> Precedence {
        match target_segment {
            Self::Start(connective) => Connective::precedence(connective),
            Self::Midway(binary) => BinaryConnective::precedence(binary),
            Self::End(connective) => Connective::precedence(connective),
        }
    }
}

/// Operator identity used to select notation and grouping. Unary operators have
/// boundary segments, while binary operators also have separators between operands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Connective {
    /// An operator with a prefix or suffix around its operand.
    Unary(UnaryConnective),

    /// An operator with a separator between operands.
    Binary(BinaryConnective),
}

impl Connective {
    /// Conjunction at [`Precedence::AND`].
    pub const AND: Self = Self::Binary(BinaryConnective::Builtin(BuiltinBinaryConnective::And));

    /// Disjunction at [`Precedence::OR`].
    pub const OR: Self = Self::Binary(BinaryConnective::Builtin(BuiltinBinaryConnective::Or));

    /// Exclusive disjunction at [`Precedence::XOR`].
    pub const XOR: Self = Self::Binary(BinaryConnective::Builtin(BuiltinBinaryConnective::Xor));

    /// Negation at [`Precedence::NOT`].
    pub const NOT: Self = Self::Unary(UnaryConnective::Builtin(BuiltinUnaryConnective::Not));
}

impl Connective {
    /// Binding strength against which a formatter compares the surrounding context.
    #[inline]
    pub const fn precedence(target_connective: Self) -> Precedence {
        match target_connective {
            Self::Unary(unary) => UnaryConnective::precedence(unary),
            Self::Binary(binary) => BinaryConnective::precedence(binary),
        }
    }
}

/// A binary operator's notation and binding strength. Custom operators may supply
/// different text for their opening, separator, and closing segments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BinaryConnective {
    /// One of the library's built-in binary operators.
    Builtin(BuiltinBinaryConnective),

    /// A custom binary operator supplied by an assertion implementation.
    Any {
        /// Optional text for this operator at the current segment.
        ///
        /// Opening, midway, and closing segments may use different text.
        target_repr: Option<&'static str>,

        /// Identity available to custom formatters that distinguish operator kinds.
        target_id: TypeId,

        /// Binding strength used when grouping this operator's operands.
        target_precedence: Precedence,
    },
}

impl BinaryConnective {
    /// Conjunction at [`Precedence::AND`].
    pub const AND: Self = Self::Builtin(BuiltinBinaryConnective::And);

    /// Disjunction at [`Precedence::OR`].
    pub const OR: Self = Self::Builtin(BuiltinBinaryConnective::Or);

    /// Exclusive disjunction at [`Precedence::XOR`].
    pub const XOR: Self = Self::Builtin(BuiltinBinaryConnective::Xor);
}

impl BinaryConnective {
    /// Binding strength used to decide whether this operator needs grouping.
    #[inline]
    pub const fn precedence(target_connective: Self) -> Precedence {
        match target_connective {
            Self::Builtin(builtin) => BuiltinBinaryConnective::precedence(builtin),
            Self::Any { target_precedence, .. } => target_precedence,
        }
    }
}

/// Binary operators understood by the supplied formatters. Their binding order
/// is disjunction, exclusive disjunction, then conjunction from weakest to strongest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BuiltinBinaryConnective {
    /// Conjunction, binding more tightly than either disjunction.
    And,

    /// Disjunction, the weakest built-in operator.
    Or,

    /// Exclusive disjunction, binding between disjunction and conjunction.
    Xor,
}

impl BuiltinBinaryConnective {
    /// Binding strength shared by the built-in human and symbolic notations.
    #[inline]
    pub const fn precedence(target_connective: Self) -> Precedence {
        match target_connective {
            Self::And => Precedence::AND,
            Self::Or => Precedence::OR,
            Self::Xor => Precedence::XOR,
        }
    }
}

/// A unary operator's notation and binding strength. Custom operators may supply
/// different text for their opening and closing segments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UnaryConnective {
    /// The library's built-in unary operator.
    Builtin(BuiltinUnaryConnective),

    /// A custom unary operator supplied by an assertion implementation.
    Any {
        /// Optional text for this operator at the current segment.
        ///
        /// Opening and closing segments may use different text.
        target_repr: Option<&'static str>,

        /// Identity available to custom formatters that distinguish operator kinds.
        target_id: TypeId,

        /// Binding strength used when grouping this operator.
        target_precedence: Precedence,
    },
}

impl UnaryConnective {
    /// Negation at [`Precedence::NOT`].
    pub const NOT: Self = Self::Builtin(BuiltinUnaryConnective::Not);
}

impl UnaryConnective {
    /// Binding strength used to decide whether this operator needs grouping.
    #[inline]
    pub const fn precedence(target_connective: Self) -> Precedence {
        match target_connective {
            Self::Builtin(builtin) => BuiltinUnaryConnective::precedence(builtin),
            Self::Any { target_precedence, .. } => target_precedence,
        }
    }
}

/// Unary operators understood by the supplied formatters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BuiltinUnaryConnective {
    /// Negation, binding more tightly than the built-in binary operators.
    Not,
}

impl BuiltinUnaryConnective {
    /// Binding strength shared by the built-in human and symbolic notations.
    #[inline]
    pub const fn precedence(target_connective: Self) -> Precedence {
        match target_connective {
            Self::Not => Precedence::NOT,
        }
    }
}

/// Relative binding strength passed to formatters.
///
/// Larger values bind more tightly. [`None`](Self::None) marks top-level output.
/// Formatters decide how precedence affects grouping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Precedence {
    /// Top-level output with no surrounding operator context.
    #[default]
    None,

    /// A nonzero binding strength. Larger values bind more tightly.
    Level(NonZero<usize>),
}

impl Precedence {
    /// Weakest operator context, immediately above top-level output.
    pub const MIN: Self = Self::Level(NonZero::<usize>::MIN);

    /// Disjunction, the weakest built-in operator context.
    pub const OR: Self = Self::MIN;

    /// Exclusive disjunction, stronger than disjunction and weaker than conjunction.
    pub const XOR: Self = Self::increase(Self::OR);

    /// Conjunction, stronger than either disjunction and weaker than negation.
    pub const AND: Self = Self::increase(Self::XOR);

    /// Negation, the strongest built-in operator context.
    pub const NOT: Self = Self::increase(Self::AND);
}

impl Precedence {
    /// Interpret a positive integer as a binding strength, rejecting zero.
    #[inline]
    pub const fn integer(target_value: usize) -> Option<Self> {
        match NonZero::new(target_value) {
            Some(target_level) => Some(Self::Level(target_level)),
            None => None,
        }
    }

    /// Return the numeric level, mapping [`None`](Self::None) to zero.
    #[inline]
    pub const fn value(target_precedence: Self) -> usize {
        match target_precedence {
            Self::None => usize::MIN,
            Self::Level(target_level) => target_level.get(),
        }
    }

    /// Return the next stronger level, saturating at the maximum.
    ///
    /// [`None`](Self::None) advances to [`MIN`](Self::MIN). A maximum level
    /// remains at the maximum because no stronger `usize` level exists.
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
