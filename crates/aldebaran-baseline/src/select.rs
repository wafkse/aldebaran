//! The module hosting various "selection" types.
//!
//! Selection types are used to represent some relationship between two or more
//! types, such as a "or", "xor", and other such relationships. These types are
//! used to represent the various possibilities that can occur in a program.

/// A selection type that represents a choice between two types.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Either<L, R> {
    /// The left-hand side of the selection.
    Left(L),

    /// The right-hand side of the selection.
    Right(R),
}

impl<T> Either<T, T> {
    /// A convenience method to get the value of the selection, regardless of
    /// which side it is.
    ///
    /// This is only available when the left-hand side and right-hand side are
    /// the same type.
    #[inline]
    pub const fn anyhow(&self) -> &T {
        match self {
            Self::Left(value) => value,
            Self::Right(value) => value,
        }
    }
}

impl<L, R> Either<L, R> {
    /// Create a new selection type with the left-hand side value.
    #[inline]
    pub const fn left(value: L) -> Self {
        Self::Left(value)
    }

    /// Create a new selection type with the right-hand side value.
    #[inline]
    pub const fn right(value: R) -> Self {
        Self::Right(value)
    }

    /// A bi-directional monadic map operation.
    ///
    /// In other words, this applies a different function depending on which
    /// side the value is on, mapping the value to a different type.
    #[inline]
    pub fn bidirectional_map<F, G, I, J>(self, left: F, right: G) -> Either<I, J>
    where
        F: FnOnce(L) -> I,
        G: FnOnce(R) -> J,
    {
        match self {
            Self::Left(value) => Either::Left(left(value)),
            Self::Right(value) => Either::Right(right(value)),
        }
    }
}

/// A selection type that represents a choice between two types, or both.
///
/// This is similar to the [`Either`] type, but it also includes a case where
/// both the left-hand side and the right-hand side are selected.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum EitherOrBoth<L, R> {
    /// The left-hand side of the selection.
    Left(L),

    /// The right-hand side of the selection.
    Right(R),

    /// Both the left-hand side and the right-hand side of the selection.
    Both(L, R),
}

impl<L, R> EitherOrBoth<L, R> {
    /// Construct a [`EitherOrBoth`] from a 2-tuple of [`Option`]s.
    #[inline]
    pub fn maybe(pair: (Option<L>, Option<R>)) -> Option<Self> {
        match pair {
            (None, None) => None,
            _ => Some(match pair {
                (Some(left), Some(right)) => Self::Both(left, right),
                (Some(left), None) => Self::Left(left),
                (None, Some(right)) => Self::Right(right),
                _ => unreachable!(),
            }),
        }
    }

    /// An all-variant monadic map operation.
    ///
    /// In other words, this applies a different function depending on which
    /// side the value is on, mapping the value to a different type.
    #[inline]
    pub fn map<F, G, I, J, K>(self, left: F, right: G, both: K) -> EitherOrBoth<I, J>
    where
        F: FnOnce(L) -> I,
        G: FnOnce(R) -> J,
        K: FnOnce(L, R) -> (I, J),
    {
        match self {
            Self::Left(value) => EitherOrBoth::Left(left(value)),
            Self::Right(value) => EitherOrBoth::Right(right(value)),
            Self::Both(left, right) => {
                let (left, right) = both(left, right);

                EitherOrBoth::Both(left, right)
            }
        }
    }

    /// Map the selected types to their references.
    #[inline]
    pub const fn as_ref(&self) -> EitherOrBoth<&L, &R> {
        match self {
            Self::Left(value) => EitherOrBoth::Left(value),
            Self::Right(value) => EitherOrBoth::Right(value),
            Self::Both(left, right) => EitherOrBoth::Both(left, right),
        }
    }

    /// Attempt to retrieve a reference to the left value.
    pub const fn left(&self) -> Option<&L> {
        match self {
            Self::Left(value) | Self::Both(value, ..) => Some(value),
            _ => None,
        }
    }
    /// Attempt to retrieve a reference to the right value.
    pub const fn right(&self) -> Option<&R> {
        match self {
            Self::Right(value) | Self::Both(.., value) => Some(value),
            _ => None,
        }
    }

    /// Attempt to retrieve a reference to both the left and right values.
    #[inline]
    pub const fn both(&self) -> Option<(&L, &R)> {
        match self {
            Self::Both(left, right) => Some((left, right)),
            _ => None,
        }
    }
}
