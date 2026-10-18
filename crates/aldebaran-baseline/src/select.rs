//! Selection enums retain which of two alternatives produced a value, including
//! whether both alternatives are present when combining optional results.

/// A value from one of two alternatives, with its side preserved.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Either<L, R> {
    /// The value from the left alternative.
    Left(L),

    /// The value from the right alternative.
    Right(R),
}

impl<T> Either<T, T> {
    /// Borrow the selected value when both alternatives have the same type.
    #[inline]
    pub const fn anyhow(&self) -> &T {
        match self {
            Self::Left(value) => value,
            Self::Right(value) => value,
        }
    }
}

impl<L, R> Either<L, R> {
    /// Select the left alternative.
    #[inline]
    pub const fn left(value: L) -> Self {
        Self::Left(value)
    }

    /// Select the right alternative.
    #[inline]
    pub const fn right(value: R) -> Self {
        Self::Right(value)
    }

    /// Apply the function for the selected side and preserve that side in the result.
    ///
    /// The function for the unselected side is not called.
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
    pub const fn left(&self) -> Option<&L> {
        match self {
            Self::Left(value) | Self::Both(value, ..) => Some(value),
            _ => None,
        }
    }
    /// Borrow the right value when the right alternative is present.
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
