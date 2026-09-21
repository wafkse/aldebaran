#![no_std]
//! Abstraction over all primitive types.
//!
//! This crate provides a set of traits and macros for working with primitive
//! types in a generic fashion.

/// Returns the signless version of the primitive type.
///
/// Prepend the type identifier with either `u` or `i` to get the unsigned or
/// signed version of the primitive type respectively.
///
/// Optionally allows to invoke the target macro with an additional token tree,
/// which is expanded as:
///
/// ```rust,ignore
/// $target_macro!($lead_tt as <...>);
/// ```
///
/// where `<...>` is a placeholder for the tokens to be emitted.
#[macro_export]
macro_rules! integer_list {
    ($target_macro:path $(=> $lead_tt:tt)?) => {
        $target_macro!(
            $($lead_tt as)?
            8 16 32 64 128 size
        );
    };
}

/// Expands to all integer postfixes that have a deterministic size.
///
/// For example, this will expand to the same as [`integer_list!`] but without
/// the `size` postfix.
#[macro_export]
macro_rules! sized_list {
    ($target_macro:path $(=> $lead_tt:tt)?) => {
        $target_macro!(
            $($lead_tt as)?
            8 16 32 64 128
        );
    };
}

/// Implements the primitive macros.
macro_rules! impl_primitive_macros {
    (
        $(
            $target_postpend:tt
        )*
    ) => {
        ::tokel::stream! {
            /// A macro for generating a list of primitive types.
            #[macro_export]
            macro_rules! primitive_list {
                ($target_macro:path => $lead_tt:tt) => {
                    $target_macro!(
                        $lead_tt as
                        $(
                            [< u $target_postpend >]:to_string:flatten:concatenate:unstringify
                            [< i $target_postpend >]:to_string:flatten:concatenate:unstringify
                        )*

                    );
                };
                ($target_macro:path) => {
                    $target_macro!(
                        $(
                            [< u $target_postpend >]:to_string:flatten:concatenate:unstringify
                            [< i $target_postpend >]:to_string:flatten:concatenate:unstringify
                        )*
                    );
                };
            }

            /// Expands to the signed variant of an unsigned primitive.
            #[macro_export]
            macro_rules! signed {
                $(
                    ([< u $target_postpend >]:to_string:flatten:concatenate:unstringify) => {
                        [< i $target_postpend >]:to_string:flatten:concatenate:unstringify
                    };
                )*

                // Previous braces are exhaustive over available unsigned types.
                // So, we can safely assume that the input is signed.
                // The signed countervariant of a signed primitive is themselves.
                ($target_type:ty) => { $target_type };
            }

            /// Expands to the unsigned variant of a signed primitive.
            #[macro_export]
            macro_rules! unsigned {
                $(
                    ([< i $target_postpend >]:to_string:flatten:concatenate:unstringify) => {
                        [< u $target_postpend >]:to_string:flatten:concatenate:unstringify
                    };
                )*

                // Previous expand options are exhaustive over available signed types.
                // So, we can safely assume that the input is unsigned.
                // The unsigned countervariant of a unsigned primitive is themselves.
                ($target_type:ty) => { $target_type };
            }

            /// Expands to a boolean literal which denotes whether the type is signed or not.
            #[macro_export]
            macro_rules! is_signed {
                $(
                    ([< i $target_postpend >]:to_string:flatten:concatenate:unstringify) => { true };
                    ([< u $target_postpend >]:to_string:flatten:concatenate:unstringify) => { false };
                )*
            }

            /// A macro for generating a list of signed primitive types.
            #[macro_export]
            macro_rules! signed_list {
                ($target_macro:path => $lead_tt:tt) => {
                    $target_macro!(
                        $lead_tt as
                        $(
                            [< i $target_postpend >]:to_string:flatten:concatenate:unstringify
                        )*
                    );
                };
                ($target_macro:path) => {
                    $target_macro!(
                        $(
                            [< i $target_postpend >]:to_string:flatten:concatenate:unstringify
                        )*
                    );
                };
            }

            /// A macro for generating a list of unsigned primitive types.
            #[macro_export]
            macro_rules! unsigned_list {
                ($target_macro:path => $lead_tt:tt) => {
                    $target_macro!(
                        $lead_tt as
                        $(
                            [< u $target_postpend >]:to_string:flatten:concatenate:unstringify
                        )*
                    );
                };
                ($target_macro:path) => {
                    $target_macro!(
                        $(
                            [< u $target_postpend >]:to_string:flatten:concatenate:unstringify
                        )*
                    );
                };
            }

        }
    }
}

integer_list!(impl_primitive_macros);
