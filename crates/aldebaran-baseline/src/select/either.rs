//! A value selected from one of two alternatives.

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
    /// side the value is on, mapping the value to a different type. The function
    /// for the unselected side is not called.
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
