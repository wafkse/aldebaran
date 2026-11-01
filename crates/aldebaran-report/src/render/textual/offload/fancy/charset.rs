//! Glyph configuration for styled diagnostic output.
//!
//! See [`Charset`], [`Delimit`], and [`Indicator`] for further information.
//!
//! Glyph selection is independent of color.

pub mod indicate;
pub mod shape;

pub use self::{indicate::Indicator, shape::Delimit};

use crate::render::textual::offload::fancy::setting::Fancyness;

/// Characters for framing source excerpts and marking annotated spans.
///
/// A `Charset` combines frame glyphs with directional cues and marker glyphs.
/// Its ASCII and Unicode presets share the same marker sequence, so changing the
/// preset affects framing and cues without changing span markers. Custom glyphs
/// are accepted without checking their terminal-cell width.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
// NOTE(invariant): None.
pub struct Charset {
    /// The subset of characters to be used as spatial delimiters.
    ///
    /// In other words, this set is primarily used to shape the error report.
    delimit: Delimit,
    /// The subset of characters to be used as spatial indicators.
    ///
    /// This includes the characters used to emphasize the importance of a
    /// particular part of the error report.
    indicator: Indicator,
}

impl Charset {
    /// ASCII framing and cues with a repeating ASCII marker sequence.
    pub const ASCII: Self = Self::tuple((Delimit::ASCII, Indicator::ASCII));

    /// Unicode framing and cues with the same ASCII markers as [`Self::ASCII`].
    pub const UNICODE: Self = Self::tuple((Delimit::UNICODE, Indicator::UNICODE));

    /// Create a new [`Charset`] with the standard set of characters.
    ///
    /// Currently, this is the [`ASCII`](Self::ASCII) charset.
    #[inline]
    pub const fn standard() -> Self {
        Self::ASCII
    }

    /// Use Unicode for [`Fancyness::Outblown`] and ASCII for the other presets.
    ///
    /// The preset's color permission is independent of this glyph selection.
    #[inline]
    pub const fn fancyness(level: Fancyness) -> Self {
        if level.unicode() { Self::UNICODE } else { Self::ASCII }
    }
}

impl Charset {
    /// Create a new [`Charset`] from a 2-element tuple.
    #[inline]
    pub const fn tuple((delimit, indicator): (Delimit, Indicator)) -> Self {
        Self { delimit, indicator }
    }

    /// Configure framing independently of annotation and directional cues.
    #[inline]
    pub const fn raw(delimit: Delimit, indicator: Indicator) -> Self {
        Self { delimit, indicator }
    }

    /// Retrieve the subset of characters to be used as [`spatial
    /// delimiters`](shape::Delimit), i.e the characters used to outline the
    /// shape of the error report.
    #[inline]
    pub const fn delimit(&self) -> Delimit {
        let &Self { delimit, .. } = self;

        delimit
    }

    /// Retrieve an immutable reference to the subset of characters to be
    /// used as [`spatial delimiters`](shape::Delimit), i.e the characters used
    /// to outline the shape of the error report.
    #[inline]
    pub const fn delimit_ref(&self) -> &Delimit {
        let &Self { ref delimit, .. } = self;

        delimit
    }

    /// Retrieve a *(mutable reference)* to the subset of characters to be
    /// used as [`spatial delimiters`](shape::Delimit), i.e the characters used
    /// to outline the shape of the error report.
    ///
    /// Useful when in-place mutation is required.
    #[inline]
    pub const fn delimit_mut(&mut self) -> &mut Delimit {
        let &mut Self { ref mut delimit, .. } = self;

        delimit
    }

    /// Retrieve the subset of characters to be used as [`spatial
    /// indicators`](indicate::Indicator), i.e the characters used to emphasize
    /// the importance of a particular part of the error report.
    #[inline]
    pub const fn indicator(&self) -> Indicator {
        let &Self { indicator, .. } = self;

        indicator
    }

    /// Retrieve an immutable reference to the subset of characters to be
    /// used as [`spatial indicators`](indicate::Indicator), i.e the characters
    /// used to emphasize the importance of a particular part of the error
    /// report.
    #[inline]
    pub const fn indicator_ref(&self) -> &Indicator {
        let &Self { ref indicator, .. } = self;

        indicator
    }

    /// Retrieve a *(a mutable reference)* to the subset of characters to be
    /// used as [`spatial indicators`](indicate::Indicator), i.e the characters
    /// used to emphasize the importance of a particular part of the error
    /// report.
    ///
    /// Useful when in-place mutation is required.
    #[inline]
    pub const fn indicator_mut(&mut self) -> &mut Indicator {
        let &mut Self { ref mut indicator, .. } = self;

        indicator
    }
}

impl Default for Charset {
    #[inline]
    fn default() -> Self {
        Self::standard()
    }
}
