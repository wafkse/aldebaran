//! Non-consuming access to tokens at offsets from the next read.
//!
//! [`Lookahead`] uses zero-based offsets, while the fixed `Lookahead1` through
//! `Lookahead32` capabilities use one-based positions and include access to all
//! preceding positions.

use super::TokenStream;

/// Inspect upcoming tokens without committing their consumption.
/// Source failures remain distinct from unavailable positions, as in [`TokenStream`].
pub trait Lookahead: TokenStream {
    /// Inspect the token `N` positions after the next token.
    ///
    /// `N = 0` selects the next token. A source error encountered at or before the
    /// requested position is returned as `Err`.
    fn lookahead<const N: usize>(&self) -> Result<Option<Self::Token>, Self::Error>;
}

impl<S> Lookahead for &mut S
where
    S: Lookahead + ?Sized,
{
    #[inline]
    fn lookahead<const N: usize>(&self) -> Result<Option<Self::Token>, Self::Error> {
        S::lookahead::<N>(self)
    }
}

/// Fixed inspection capabilities that include all preceding token positions.
macro_rules! lookahead {
    () => {};
    (impl for []) => {};
    (
        impl for [
            $target_index:literal

            $($target_rest:literal)*
        ]
    ) => {
        tokel::stream! {
            #[doc = concat!("Non-consuming access through token position ", stringify!($target_index), ", counting the next token as position 1.")]
            pub trait [< Lookahead $target_index >]:to_string:flatten:concatenate:unstringify
            where
                Self: TokenStream $(
                    + [< Lookahead $target_rest >]:to_string:flatten:concatenate:unstringify
                )*
            {
                #[doc = concat!("Inspect token position ", stringify!($target_index), " without consuming it, counting the next token as position 1. Source failures at or before that position remain errors.")]
                fn [< lookahead $target_index >]:to_string:flatten:concatenate:unstringify(
                    &self,
                ) -> Result<Option<Self::Token>, Self::Error>;
            }

            impl<T> [< Lookahead $target_index >]:to_string:flatten:concatenate:unstringify for T
            where
                T: Lookahead + ?Sized,
            {
                #[inline]
                fn [< lookahead $target_index >]:to_string:flatten:concatenate:unstringify(
                    &self,
                ) -> Result<Option<Self::Token>, Self::Error> {
                    self.lookahead::<{ $target_index - 1 }>()
                }
            }
        }

        lookahead!(impl for [$($target_rest)*]);
    };
    (
        $target_start:literal..$target_end:literal
    ) => {
        tokel::stream! {
            lookahead!(impl for [[< >]:sequence[[ $target_start..=$target_end ]]:reverse]);
        }
    };
}

lookahead!(1..32);

#[cfg(test)]
mod tests {
    use super::{Lookahead1, Lookahead32};
    use crate::stream::storage::Store;

    #[test]
    fn recursive_fixed_lookahead_traits_preserve_positions() {
        static TOKENS: [u8; 32] = [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
        ];

        let stream = Store::new(&TOKENS);

        assert_eq!(stream.lookahead1(), Ok(Some(0)));
        assert_eq!(stream.lookahead32(), Ok(Some(31)));
    }
}
