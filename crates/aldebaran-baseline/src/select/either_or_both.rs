//! Values selected from one or both of two alternatives.

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
    /// Combine two optional values, returning [`None`] only when both are absent.
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
    /// side the value is on, mapping the value to a different type. Only the
    /// function for the current variant is called.
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

    /// Borrow the left value when the left alternative is present.
    #[inline]
    pub const fn left(&self) -> Option<&L> {
        match self {
            Self::Left(value) | Self::Both(value, ..) => Some(value),
            _ => None,
        }
    }
    /// Borrow the right value when the right alternative is present.
    #[inline]
    pub const fn right(&self) -> Option<&R> {
        match self {
            Self::Right(value) | Self::Both(.., value) => Some(value),
            _ => None,
        }
    }

    /// Borrow both values only when both alternatives are present.
    #[inline]
    pub const fn both(&self) -> Option<(&L, &R)> {
        match self {
            Self::Both(left, right) => Some((left, right)),
            _ => None,
        }
    }
}
