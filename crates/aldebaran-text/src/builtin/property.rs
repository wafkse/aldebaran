//! Property assertions for byte and character components.
//!
//! See [`AsciiDigit`], [`Whitespace`], and [`Digit`] for further information.
//!
//! Value and reference inputs follow the same acceptance rules, and descriptions use the
//! property name in angle brackets.

use core::fmt;

use aldebaran_logic::{
    fmt::Precedence,
    prelude::{Assert, Formatter},
};

use aldebaran_print::prelude::Print;

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

mod detail {
    //! Implementation details for built-in character properties.

    /// Determines whether `target_codepoint` is horizontal whitespace.
    ///
    /// [UTS #18, Annex C] defines horizontal whitespace as `U+0009` plus every
    /// code point whose `General_Category` is `Space_Separator`. In Unicode
    /// 18.0.0, the set is:
    ///
    /// - `U+0009` CHARACTER TABULATION
    /// - `U+0020` SPACE
    /// - `U+00A0` NO-BREAK SPACE
    /// - `U+1680` OGHAM SPACE MARK
    /// - `U+2000`-`U+200A` EN QUAD through HAIR SPACE
    /// - `U+202F` NARROW NO-BREAK SPACE
    /// - `U+205F` MEDIUM MATHEMATICAL SPACE
    /// - `U+3000` IDEOGRAPHIC SPACE
    ///
    /// The separator categories come from Unicode 18.0.0 [`UnicodeData.txt`].
    ///
    /// [UTS #18, Annex C]: https://www.unicode.org/reports/tr18/tr18-25.html#Compatibility_Properties
    /// [`UnicodeData.txt`]: https://www.unicode.org/Public/18.0.0/ucd/UnicodeData.txt
    #[inline]
    pub const fn is_horizontal_whitespace(target_codepoint: char) -> bool {
        if let '\t' | ' ' | '\u{00A0}' | '\u{1680}' | '\u{2000}'..='\u{200A}' | '\u{202F}' | '\u{205F}' | '\u{3000}' = target_codepoint {
            true
        } else {
            false
        }
    }

    /// Determines whether `target_codepoint` is vertical whitespace.
    ///
    /// These are the `White_Space` code points not included in the horizontal
    /// whitespace definition from [UTS #18, Annex C]:
    ///
    /// - `U+000A` LINE FEED (LF)
    /// - `U+000B` LINE TABULATION (VT)
    /// - `U+000C` FORM FEED (FF)
    /// - `U+000D` CARRIAGE RETURN (CR)
    /// - `U+0085` NEXT LINE (NEL)
    /// - `U+2028` LINE SEPARATOR
    /// - `U+2029` PARAGRAPH SEPARATOR
    ///
    /// The set is listed in Unicode 18.0.0 [`PropList.txt`].
    ///
    /// [UTS #18, Annex C]: https://www.unicode.org/reports/tr18/tr18-25.html#Compatibility_Properties
    /// [`PropList.txt`]: https://www.unicode.org/Public/18.0.0/ucd/PropList.txt
    #[inline]
    pub const fn is_vertical_whitespace(target_codepoint: char) -> bool {
        if let '\n' | '\u{000B}' | '\u{000C}' | '\r' | '\u{0085}' | '\u{2028}' | '\u{2029}' = target_codepoint {
            true
        } else {
            false
        }
    }
}

property!(
    /// A character-property assertion implementing [`Assert<char>`] and
    /// [`Assert<&char>`].
    ///
    /// It accepts `U+0009` and the Unicode 18.0.0 `Space_Separator` code points:
    /// `U+0020`, `U+00A0`, `U+1680`, `U+2000`–`U+200A`, `U+202F`, `U+205F`, and
    /// `U+3000`. This set follows [UTS #18, Annex C] and [`UnicodeData.txt`].
    ///
    /// [UTS #18, Annex C]: https://www.unicode.org/reports/tr18/tr18-25.html#Compatibility_Properties
    /// [`UnicodeData.txt`]: https://www.unicode.org/Public/18.0.0/ucd/UnicodeData.txt
    pub HorizontalWhitespace where {
        char as detail::is_horizontal_whitespace;
    }
);

property!(
    /// A character-property assertion implementing [`Assert<char>`] and
    /// [`Assert<&char>`].
    ///
    /// It accepts the Unicode 18.0.0 vertical whitespace code points listed in
    /// [`PropList.txt`]:
    ///
    /// - `U+000A` LINE FEED (LF)
    /// - `U+000B` LINE TABULATION (VT)
    /// - `U+000C` FORM FEED (FF)
    /// - `U+000D` CARRIAGE RETURN (CR)
    /// - `U+0085` NEXT LINE (NEL)
    /// - `U+2028` LINE SEPARATOR
    /// - `U+2029` PARAGRAPH SEPARATOR
    ///
    /// [`PropList.txt`]: https://www.unicode.org/Public/18.0.0/ucd/PropList.txt
    pub VerticalWhitespace where {
        char as detail::is_vertical_whitespace;
    }
);

#[cfg(test)]
mod tests {
    use super::*;

    use alloc::string::String;

    use aldebaran_logic::{fmt::common::Human, prelude::Assert};

    macro_rules! assert_property {
        ($target:ident, $byte_match:expr, $byte_miss:expr, $char_match:expr, $char_miss:expr) => {{
            let assertion = $target;
            let byte_match: u8 = $byte_match;
            let byte_miss: u8 = $byte_miss;
            let char_match: char = $char_match;
            let char_miss: char = $char_miss;

            assert!(Assert::<u8>::assert(&assertion, byte_match));
            assert!(Assert::<&u8>::assert(&assertion, &byte_match));
            assert!(!Assert::<u8>::assert(&assertion, byte_miss));
            assert!(!Assert::<&u8>::assert(&assertion, &byte_miss));

            assert!(Assert::<char>::assert(&assertion, char_match));
            assert!(Assert::<&char>::assert(&assertion, &char_match));
            assert!(!Assert::<char>::assert(&assertion, char_miss));
            assert!(!Assert::<&char>::assert(&assertion, &char_miss));

            let mut output = String::new();
            let result = Assert::<u8>::output::<_, Human>(&assertion, &mut output);

            assert!(result.is_ok());
            assert_eq!(output, concat!("<", stringify!($target), ">"));
        }};
    }

    #[test]
    fn ascii_properties_match_ascii_boundaries() {
        assert_property!(Ascii, b'A', 0x80, 'A', 'é');
        assert_property!(AsciiDigit, b'7', b'G', '7', '٧');
        assert_property!(AsciiAlphabetic, b'Z', b'5', 'z', 'é');
        assert_property!(AsciiAlphanumeric, b'9', b'_', 'A', '_');
        assert_property!(AsciiWhitespace, b'\t', b'\x0B', '\n', '\u{00A0}');
        assert_property!(AsciiControl, b'\0', b'A', '\x7F', 'A');
        assert_property!(AsciiGraphical, b'!', b' ', '~', ' ');
        assert_property!(AsciiPunctuation, b'!', b'a', '_', 'a');
        assert_property!(AsciiHexadecimal, b'f', b'g', 'F', 'G');
    }

    #[test]
    fn unicode_properties_distinguish_byte_and_character_rules() {
        assert_property!(Whitespace, b'\t', b'A', '\u{2003}', 'A');
        assert_property!(Alphabetic, b'Z', b'1', 'é', '1');
        assert_property!(Alphanumeric, b'0', b'_', 'é', '_');
        assert_property!(Digit, b'9', b'G', '½', 'A');
    }
}
