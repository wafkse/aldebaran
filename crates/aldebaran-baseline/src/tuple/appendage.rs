//! Tuple growth that preserves component types and positions.

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
