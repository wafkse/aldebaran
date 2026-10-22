//! A value selected from one of two alternatives.

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
