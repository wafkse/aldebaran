//! Reusable character and byte property assertions for text traversal.
//!
//! Property assertions wrap standard ASCII and Unicode classification functions
//! in the `Assert` interface. The generated zero-sized types can therefore be
//! composed with logical connectives and used directly by [`Text`](super::Text).

use core::fmt;

use aldebaran_logic::{
    fmt::Precedence,
    prelude::{Assert, Formatter, Input},
};

use aldebaran_print::prelude::Print;

/// An [`Assert`] implementation that always returns `true`.
#[derive(Debug, Clone, Copy, Hash, Default)]
pub struct Anything;

impl<I> Assert<I> for Anything
where
    I: Input,
{
    #[inline]
    fn assert(&self, _: I) -> bool {
        true
    }

    #[inline]
    fn output_with<W, F>(target_value: &Self, writer: &mut W, _: Precedence) -> fmt::Result
    where
        W: fmt::Write,
        F: Formatter,
    {
        Self::print(target_value, writer)
    }
}

impl Print for Anything {
    type Context = ();
    #[inline]
    fn print_with_ctx<W>(&self, writer: &mut W, _: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        write!(writer, "<anything>")
    }
}

/// A tt-muncher that expands to [`Assert`] implementations for a target type.
macro_rules! property {
    (
        @impl for $target_ident:ident where {}
    ) => {/* do nothing */};
    (
        @impl for $target_ident:ident where {
            $target_ty:ty as $target_path:path;

            $(
                $target_rest_tt:tt
            )*
        }
    ) => {
        impl Assert<$target_ty> for $target_ident {
            #[inline]
            fn assert(&self, input: $target_ty) -> bool {
                $target_path(input)
            }

            #[inline]
            fn output_with<W, F>(target_value: &Self, writer: &mut W, _: Precedence) -> fmt::Result
            where
                W: fmt::Write,
                F: Formatter,
            {
                Self::print(target_value, writer)
            }
        }

        impl Assert<&$target_ty> for $target_ident
        where $target_ty: Copy
        {
            #[inline]
            fn assert(&self, input: &$target_ty) -> bool {
                $target_path(*input)
            }

            #[inline]
            fn output_with<W, F>(target_value: &Self, writer: &mut W, _: Precedence) -> fmt::Result
            where
                W: fmt::Write,
                F: Formatter,
            {
                Self::print(target_value, writer)
            }
        }

        property!(
            @impl for $target_ident where {
                $(
                    $target_rest_tt
                )*
            }
        );
    };

    (
        @impl for $target_ident:ident where {
            ref $target_ty:ty as $target_path:path;

            $(
                $target_rest_tt:tt
            )*
        }
    ) => {
        impl Assert<$target_ty> for $target_ident
        {
            #[inline]
            fn assert(&self, input: $target_ty) -> bool {
                $target_path(&input)
            }

            #[inline]
            fn output_with<W, F>(target_value: &Self, writer: &mut W, _: Precedence) -> fmt::Result
            where
                W: fmt::Write,
                F: Formatter,
            {
                Self::print(target_value, writer)
            }
        }

        impl Assert<&$target_ty> for $target_ident
        {
            #[inline]
            fn assert(&self, input: &$target_ty) -> bool {
                $target_path(input)
            }

            #[inline]
            fn output_with<W, F>(target_value: &Self, writer: &mut W, _: Precedence) -> fmt::Result
            where
                W: fmt::Write,
                F: Formatter,
            {
                Self::print(target_value, writer)
            }
        }

        property!(
            @impl for $target_ident where {
                $(
                    $target_rest_tt
                )*
            }
        );
    };
    (
        $(
            #[$target_meta:meta]
        )*

        $target_vis:vis $target_ident:ident

        $(
            where {
                $(
                    $target_bound_tt:tt
                )*
            }
        )?
    ) => {
            #[doc = concat!("Asserts that the input satisfies the property `<", stringify!($target_ident), ">`.")]
            #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
            $(#[$target_meta])*
            $target_vis struct $target_ident;

            impl Print for $target_ident {
                type Context = ();

                #[inline]
                fn print_with_ctx<W>(
                    &self,
                    writer: &mut W,
                    _: &Self::Context,
                ) -> core::fmt::Result
                where
                    W: core::fmt::Write,
                {
                    write!(writer, "<{}>", stringify!($target_ident))
                }
            }

        $(
            property!(
                @impl for $target_ident where {
                    $(
                        $target_bound_tt
                    )*
                }
            );
        )?
    };
}

property!(
    pub Ascii where {
        ref u8 as u8::is_ascii;
        ref char as char::is_ascii;
    }
);

property!(
    pub AsciiDigit where {
        ref u8 as u8::is_ascii_digit;
        ref char as char::is_ascii_digit;
    }
);

property!(
    pub AsciiAlphabetic where {
        ref u8 as u8::is_ascii_alphabetic;
        ref char as char::is_ascii_alphabetic;
    }
);

property!(
    pub AsciiAlphanumeric where {
        ref u8 as u8::is_ascii_alphanumeric;
        ref char as char::is_ascii_alphanumeric;
    }
);

property!(
    pub AsciiWhitespace where {
        ref u8 as u8::is_ascii_whitespace;
        ref char as char::is_ascii_whitespace;
    }
);

property!(
    pub AsciiControl where {
        ref u8 as u8::is_ascii_control;
        ref char as char::is_ascii_control;
    }
);

property!(
    pub AsciiGraphical where {
        ref u8 as u8::is_ascii_graphic;
        ref char as char::is_ascii_graphic;
    }
);

property!(
    pub AsciiPunctuation where {
        ref u8 as u8::is_ascii_punctuation;
        ref char as char::is_ascii_punctuation;
    }
);

property!(
    pub AsciiHexadecimal where {
        ref u8 as u8::is_ascii_hexdigit;
        ref char as char::is_ascii_hexdigit;
    }
);

property!(
    pub Whitespace where {
        ref u8 as u8::is_ascii_whitespace;
        char as char::is_whitespace;
    }
);

property!(
    pub Alphabetic where {
        ref u8 as u8::is_ascii_alphabetic;
        char as char::is_alphabetic;
    }
);

property!(
    pub Alphanumeric where {
        ref u8 as u8::is_ascii_alphanumeric;
        char as char::is_alphanumeric;
    }
);

property!(
    pub Digit where {
        ref u8 as u8::is_ascii_digit;
        char as char::is_numeric;
    }
);
