//! Type-level tuple append and prepend operations.
//!
//! [`Appendage`] grows tuples while preserving their component types, and
//! [`Uniform`] converts homogeneous tuples into fixed-size arrays.

/// Tuple operations that preserve each element's type and position.
///
/// Implementations are provided for tuples with one through seventeen elements.
pub trait Appendage {
    /// The tuple type with `T` as its final element.
    type Append<T>;

    /// The tuple type with `T` as its first element.
    type Prepend<T>;

    /// Number of elements in the current tuple.
    const LENGTH: usize;

    /// Return the tuple with `target_value` appended.
    fn append<T>(self, target_value: T) -> Self::Append<T>;

    /// Return the tuple with `target_value` prepended.
    fn prepend<T>(self, target_value: T) -> Self::Prepend<T>;
}

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

/// Generate [`Appendage`] implementations for the supplied tuple shapes.
macro_rules! appendage {
    (
        @

        (
            $(
                $target_ty:ident
            ),+

            $(
                ,
            )?
        )

        $(,)?
    ) => {};
    (
        @

        (
            $(
                $target_ty:ident
            ),+

            $(
                ,
            )?
        ),

        (
            $(
                $target_next_ty:ident
            ),+

            $(
                ,
            )?
        )

        $(
            $target_tt:tt
        )*
    ) => {
        tokel::stream! {
            impl<$($target_ty),+> $crate::tuple::Appendage for ($($target_ty),+,) {
                type Append<T> = (
                    $($target_ty),+, T,
                );

                type Prepend<T> = (
                    T, $($target_ty),+,
                );

                const LENGTH: usize = [< $($target_ty)+ >]:count;

                #[inline]
                fn append<T>(self, target_value: T) -> Self::Append<T> {
                    let (
                        $(
                            [< $target_ty >]:case[[lower]],
                        )+
                    ) = self;

                    (
                        $(
                            [< $target_ty >]:case[[lower]],
                        )+ target_value
                    )
                }

                #[inline]
                fn prepend<T>(self, target_value: T) -> Self::Prepend<T> {
                    let (
                        $(
                            [< $target_ty >]:case[[lower]],
                        )+
                    ) = self;

                    (
                        target_value,
                        $(
                            [< $target_ty >]:case[[lower]],
                        )+
                    )
                }
            }
        }

        appendage! {
            @

            (
                $(
                    $target_next_ty
                ),+
                ,
            )

            $(
                $target_tt
            )*
        }
    };
    (
        $(
            (
                $(
                    $target_ty:ident
                ),+

                $(
                    ,
                )?
            )
        ),+

        $(,)?
    ) => {
        appendage! {
            @

            $(
                (
                    $(
                        $target_ty
                    ),+
                )
            ),+
        }
    };
}

appendage!(
    (T0,),
    (T0, T1),
    (T0, T1, T2),
    (T0, T1, T2, T3),
    (T0, T1, T2, T3, T4),
    (T0, T1, T2, T3, T4, T5),
    (T0, T1, T2, T3, T4, T5, T6),
    (T0, T1, T2, T3, T4, T5, T6, T7),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16),
);

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
