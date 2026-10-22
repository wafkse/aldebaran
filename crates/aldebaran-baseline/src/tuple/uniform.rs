//! Homogeneous tuple conversion to fixed-size arrays.

use super::Appendage;

/// Conversion from a homogeneous tuple to an array of the same length.
///
/// Implementations are provided for tuple lengths one through sixteen.
pub trait Uniform<const K: usize, T>
where
    Self: Appendage,
{
    /// Consume the tuple into an array, preserving element order.
    fn array(self) -> [T; K];
}

/// Generate [`Uniform`] implementations for homogeneous tuple lengths.
macro_rules! uniform {
    () => {};
    (
        for use $target_size:literal [
            $(
                $target_index:literal
            )+
        ]
    ) => {
        tokel::stream! {
            #[inline]
            fn array(self) -> [T; $target_size] {
                let (
                    $(
                        [< t $target_index >]:to_string:flatten:concatenate:unstringify,
                    )+
                ) = self;

                [
                    $(
                        [< t $target_index >]:to_string:flatten:concatenate:unstringify
                    ),+
                ]
            }
        }
    };

    (
        @ [

        ]
    ) => {};
    (
        @ [
            $target_current:literal

            $(
                $target_next:literal
            )*
        ]
    ) => {
        tokel::stream! {
            impl<T> Uniform<$target_current, T> for ([< T, >]:repeat[[ $target_current ]]) {
                uniform!(for use $target_current [[<>]:sequence[[ 1..=$target_current ]]]);
            }

            uniform!(@ [$($target_next)*]);
        }
    };
    (
        $target_start:literal..$target_end:literal
    ) => {
        tokel::stream! {
            uniform!(@ [[<>]:sequence[[ $target_start..=$target_end ]]]);
        }
    };
}

uniform!(1..16);

#[cfg(test)]
mod tests {
    use super::Uniform;

    #[test]
    fn tuple_conversion_preserves_order_at_supported_length_boundaries() {
        assert_eq!((7,).array(), [7]);
        assert_eq!(
            (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15).array(),
            [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]
        );
    }
}
