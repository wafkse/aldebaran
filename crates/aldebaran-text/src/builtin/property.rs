//! Property assertions for byte and character components.
//!
//! ASCII-prefixed properties use ASCII rules for both component types. Other
//! properties use ASCII rules for bytes and Unicode predicates for `char`.
//! For example, [`Digit`] uses `char::is_numeric` for characters and an ASCII
//! digit check for bytes. Value and reference inputs have identical acceptance
//! rules, and descriptions use the property name in angle brackets.

use core::fmt;

use aldebaran_logic::{
    fmt::Precedence,
    prelude::{Assert, Formatter},
};

use aldebaran_print::prelude::Print;

/// Property declarations share acceptance rules across values and references.
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
            #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
            $(#[$target_meta])*
            $target_vis struct $target_ident;

            impl Print for $target_ident {
                type Context = ();

                #[inline]
                fn print_with_context<W>(
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
    /// Accepts code points from `0x00` through `0x7f` in either source kind.
    pub Ascii where {
        ref u8 as u8::is_ascii;
        ref char as char::is_ascii;
    }
);

property!(
    /// Accepts only `0` through `9`, including for Unicode character sources.
    pub AsciiDigit where {
        ref u8 as u8::is_ascii_digit;
        ref char as char::is_ascii_digit;
    }
);

property!(
    /// Accepts `A` through `Z` and `a` through `z` in either source kind.
    pub AsciiAlphabetic where {
        ref u8 as u8::is_ascii_alphabetic;
        ref char as char::is_ascii_alphabetic;
    }
);

property!(
    /// Accepts ASCII letters and decimal digits, excluding underscores and non-ASCII letters.
    pub AsciiAlphanumeric where {
        ref u8 as u8::is_ascii_alphanumeric;
        ref char as char::is_ascii_alphanumeric;
    }
);

property!(
    /// Accepts space, tab, line feed, form feed, and carriage return, excluding vertical tab.
    pub AsciiWhitespace where {
        ref u8 as u8::is_ascii_whitespace;
        ref char as char::is_ascii_whitespace;
    }
);

property!(
    /// Accepts ASCII control codes `0x00` through `0x1f` and `0x7f`.
    pub AsciiControl where {
        ref u8 as u8::is_ascii_control;
        ref char as char::is_ascii_control;
    }
);

property!(
    /// Accepts printable ASCII except space, covering `0x21` through `0x7e`.
    pub AsciiGraphical where {
        ref u8 as u8::is_ascii_graphic;
        ref char as char::is_ascii_graphic;
    }
);

property!(
    /// Accepts ASCII graphic characters other than letters and digits.
    pub AsciiPunctuation where {
        ref u8 as u8::is_ascii_punctuation;
        ref char as char::is_ascii_punctuation;
    }
);

property!(
    /// Accepts `0` through `9`, `A` through `F`, and `a` through `f` in either source kind.
    pub AsciiHexadecimal where {
        ref u8 as u8::is_ascii_hexdigit;
        ref char as char::is_ascii_hexdigit;
    }
);

property!(
    /// Uses [`char::is_whitespace`] for Unicode scalars and [`u8::is_ascii_whitespace`]
    /// for bytes, so character sources can accept additional whitespace.
    pub Whitespace where {
        ref u8 as u8::is_ascii_whitespace;
        char as char::is_whitespace;
    }
);

property!(
    /// Uses [`char::is_alphabetic`] for Unicode scalars and ASCII letters for bytes.
    pub Alphabetic where {
        ref u8 as u8::is_ascii_alphabetic;
        char as char::is_alphabetic;
    }
);

property!(
    /// Uses [`char::is_alphanumeric`] for Unicode scalars and ASCII letters or digits for bytes.
    pub Alphanumeric where {
        ref u8 as u8::is_ascii_alphanumeric;
        char as char::is_alphanumeric;
    }
);

property!(
    /// Uses [`char::is_numeric`] for Unicode scalars, including numeric characters
    /// beyond decimal digits, but accepts only `0` through `9` for bytes.
    pub Digit where {
        ref u8 as u8::is_ascii_digit;
        char as char::is_numeric;
    }
);
