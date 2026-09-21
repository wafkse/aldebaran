//! Character sets used to draw fancy textual diagnostics.
//!
//! [`Charset`] groups the glyphs required for source guides, annotation markers,
//! and report framing. Indicator and delimiter policies are kept separate so
//! themes can change appearance without changing line layout logic.

pub mod indicate;
pub mod shape;

pub use self::{indicate::Indicator, shape::Delimit};

use crate::render::textual::offload::fancy::setting::Fancyness;

/// Glyph collection used by the fancy textual renderer.
///
/// The value carries all characters required by source guides, connection lines,
/// and annotation markers. Rendering code reads the collection as policy and does
/// not embed terminal glyph choices in layout decisions.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
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
    /// A stricly *ASCII* character set.
    pub const ASCII: Self = Self::tuple((Delimit::ASCII, Indicator::ASCII));

    /// An *unicode* character set.
    pub const UNICODE: Self = Self::tuple((Delimit::UNICODE, Indicator::UNICODE));

    /// Create a new [`Charset`] with the standard set of characters.
    ///
    /// Currently, this is the [`ASCII`](Self::ASCII) charset.
    #[inline]
    pub const fn standard() -> Self {
        Self::ASCII
    }

    /// Select a [`Charset`] based on a [`Fancyness`] level.
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

    /// Create a new [`chatset`](Charset) from its raw parts: [`Delimit`] and
    /// [`Indicator`].
    #[inline]
    pub const fn from_raw_parts(delimit: Delimit, indicator: Indicator) -> Self {
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

    /// Retrieve a *(an immutable reference)* to the subset of characters to be
    /// used as [`spatial delimiters`](shape::Delimit), i.e the characters used
    /// to outline the shape of the error report.
    ///
    /// Useful when in-place mutation is required.
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

    /// Retrieve a *(an immutable reference)* to the subset of characters to be
    /// used as [`spatial indicators`](indicate::Indicator), i.e the characters
    /// used to emphasize the importance of a particular part of the error
    /// report.
    ///
    /// Useful when in-place mutation is required.
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
