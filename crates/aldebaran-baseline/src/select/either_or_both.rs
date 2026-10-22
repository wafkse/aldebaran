//! Values selected from one or both of two alternatives.

/// The present values from two alternatives, preserving whether one or both exist.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum EitherOrBoth<L, R> {
    /// Only the left alternative is present.
    Left(L),

    /// Only the right alternative is present.
    Right(R),

    /// Both alternatives are present.
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

    /// Map the present values while preserving whether the left, right, or both were present.
    ///
    /// Only the function for the current variant is called.
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

    /// Borrow the present values without changing the variant.
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
