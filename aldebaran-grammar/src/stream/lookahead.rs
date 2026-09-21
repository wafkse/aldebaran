//! Stream lookahead utilities.
//!
//! The fixed lookahead traits are generated recursively from one inclusive range.
//! Identifier synthesis uses Tokel transformers and requires no unstable Rust features.

use super::TokenStream;

/// A token stream that supports zero-based lookahead.
pub trait Lookahead: TokenStream {
    /// Retrieve the token `N` positions ahead without consuming it.
    ///
    /// `N = 0` denotes the next token.
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

/// Generate the fixed `LookaheadN` capability chain.
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
            #[doc = concat!("Lookahead capability for the ", stringify!($target_index), " immediate token.")]
            pub trait [< Lookahead $target_index >]:to_string:flatten:concatenate:unstringify
            where
                Self: TokenStream $(
                    + [< Lookahead $target_rest >]:to_string:flatten:concatenate:unstringify
                )*
            {
                #[doc = concat!("Retrieve the ", stringify!($target_index), " immediate token without consuming it.")]
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
