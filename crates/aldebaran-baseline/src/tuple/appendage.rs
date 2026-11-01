//! Tuple growth that preserves component types and positions.

/// A trait for tuple types that can have elements appended or prepended.
///
/// This is useful for certain cumulative operations, such as appending values
/// to a tuple in a type-safe manner.
pub trait Appendage {
    /// The next type that has type `T` appended to, e.g. `(..., T)`.
    type Append<T>;

    /// The next type that has type `T` prepended to, e.g `(T, ...)`.
    type Prepend<T>;

    /// The current length for this [`Appendage`] type.
    const LENGTH: usize;

    /// Append a value of type `T` to the current [`Appendage`] type.
    fn append<T>(self, target_value: T) -> Self::Append<T>;

    /// Prepend a value of type `T` to the current [`Appendage`] type.
    fn prepend<T>(self, target_value: T) -> Self::Prepend<T>;
}

/// TT-Muncher that implements [`Appendage`] for tuple types.
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
