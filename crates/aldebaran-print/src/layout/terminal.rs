//! Selects the displayed text and cell width of an already segmented grapheme.
//!
//! [`TerminalGrapheme`] distinguishes tab expansion, control replacement, and
//! unchanged text. Its measurement uses the proposed row column, independently
//! of the original source coordinates associated with that grapheme.

use alloc::string::String;
use core::{iter, num::NonZero};

use unicode_width::UnicodeWidthStr;

/// Describes how a grapheme is displayed at a particular terminal column.
///
/// The selected form couples text emission with its cell count. A tab must be
/// classified again if wrapping moves it to a different row origin. Other text
/// retains either its original bytes or a replacement for each control scalar.
#[derive(Debug, Clone, Copy)]
pub(in crate::layout) enum TerminalGrapheme<'input> {
    /// A tab expanded to spaces up to the next configured stop.
    Tab {
        /// Number of spaces emitted at the current column.
        width: usize,
    },

    /// Text with control scalars displayed as Unicode replacement characters.
    ///
    /// Non-control scalars retain their original text. The measured width is
    /// the original grapheme's scalar count under this replacement policy.
    Replacement {
        /// Original grapheme before replacement.
        text: &'input str,

        /// Scalar count used as the replacement form's terminal width.
        width: usize,
    },

    /// Unmodified text measured with the non-CJK Unicode width policy.
    ///
    /// This form permits a row to borrow visualization bytes directly when no
    /// other grapheme in that row needs normalization.
    Verbatim {
        /// Unmodified grapheme text.
        text: &'input str,

        /// Cell count under the non-CJK width policy.
        width: usize,
    },
}

impl<'input> TerminalGrapheme<'input> {
    /// Classifies a grapheme for display beginning at `column` within its row.
    ///
    /// `text` is an already segmented grapheme. Tabs expand to the next multiple
    /// of `tab_width`, while control replacement and ordinary width follow the
    /// policies represented by the corresponding variants.
    #[inline]
    pub(in crate::layout) fn new(text: &'input str, tab_width: NonZero<usize>, column: usize) -> Self {
        let contains_control_character = text.chars().any(char::is_control);

        if text == "\t" {
            let width = tab_width.get() - column % tab_width.get();

            Self::Tab { width }
        } else if contains_control_character {
            let width = text.chars().count();

            Self::Replacement { text, width }
        } else {
            let width = text.width();

            Self::Verbatim { text, width }
        }
    }

    /// Returns the cell count measured at the column used for classification.
    #[inline]
    pub(in crate::layout) const fn width(&self) -> usize {
        match *self {
            Self::Tab { width } | Self::Replacement { width, .. } | Self::Verbatim { width, .. } => width,
        }
    }

    /// Reports whether the displayed text differs from the visualization bytes.
    #[inline]
    pub(in crate::layout) const fn transformed(&self) -> bool {
        matches!(self, Self::Tab { .. } | Self::Replacement { .. })
    }

    /// Appends the selected display form without inserting row delimiters.
    ///
    /// The caller retains ownership of `buffer` and any capacity it acquires.
    /// Source provenance and cell placement are independent of this emission.
    pub(in crate::layout) fn append(&self, buffer: &mut String) {
        match *self {
            Self::Tab { width } => buffer.extend(iter::repeat_n(' ', width)),
            Self::Replacement { text, .. } => {
                for character in text.chars() {
                    buffer.push(if character.is_control() {
                        char::REPLACEMENT_CHARACTER
                    } else {
                        character
                    });
                }
            }
            Self::Verbatim { text, .. } => buffer.push_str(text),
        }
    }
}
