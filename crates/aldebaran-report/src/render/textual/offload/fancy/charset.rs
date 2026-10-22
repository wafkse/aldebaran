//! Glyph configuration for styled diagnostic output.
//!
//! [`Charset`] pairs the frame edges, intersections, and corners in [`Delimit`]
//! with the markers and directional cues in [`Indicator`].
//! [`Fancy`](super::Fancy) uses frame glyphs around the source excerpt and a
//! repeating marker sequence to connect highlighted spans to their messages.
//! Glyph selection is independent of color.

pub mod indicate;
pub mod shape;

pub use self::{indicate::Indicator, shape::Delimit};

use crate::render::textual::offload::fancy::setting::Fancyness;

/// Framing and annotation glyphs used by the source excerpt.
///
/// ASCII and Unicode presets share the same marker sequence, so changing the
/// preset changes framing and directional cues without changing span markers.
/// Custom glyphs are accepted without checking their terminal-cell width.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
// NOTE(invariant): None.
pub struct Charset {
    /// Edges and corners that delimit the excerpt and separate its gutter from source text.
    delimit: Delimit,
    /// Marker sequence for annotation spans together with optional directional and diagnostic cues.
    indicator: Indicator,
}

impl Charset {
    /// ASCII framing and cues with a repeating ASCII marker sequence.
    pub const ASCII: Self = Self::tuple((Delimit::ASCII, Indicator::ASCII));

    /// Unicode framing and cues with the same ASCII markers as [`Self::ASCII`].
    pub const UNICODE: Self = Self::tuple((Delimit::UNICODE, Indicator::UNICODE));

    /// Use ASCII framing and indicators, as [`Default`] does.
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
    /// Pair frame glyphs with indicators in that order, allowing mixed presets.
    #[inline]
    pub const fn tuple((delimit, indicator): (Delimit, Indicator)) -> Self {
        Self { delimit, indicator }
    }

    /// Configure framing independently of annotation and directional cues.
    #[inline]
    pub const fn raw(delimit: Delimit, indicator: Indicator) -> Self {
        Self { delimit, indicator }
    }

    /// Framing used at the excerpt heading, gutter separator, and closing edge.
    #[inline]
    pub const fn delimit(&self) -> Delimit {
        let &Self { delimit, .. } = self;

        delimit
    }

    /// Edge, corner, and intersection configuration for the source excerpt.
    #[inline]
    pub const fn delimit_ref(&self) -> &Delimit {
        let &Self { ref delimit, .. } = self;

        delimit
    }

    /// Customize excerpt framing without changing annotation markers.
    #[inline]
    pub const fn delimit_mut(&mut self) -> &mut Delimit {
        let &mut Self { ref mut delimit, .. } = self;

        delimit
    }

    /// Indicators whose marker sequence is copied for each source excerpt.
    #[inline]
    pub const fn indicator(&self) -> Indicator {
        let &Self { indicator, .. } = self;

        indicator
    }

    /// Marker sequence and cue configuration, including its stored sequence position.
    #[inline]
    pub const fn indicator_ref(&self) -> &Indicator {
        let &Self { ref indicator, .. } = self;

        indicator
    }

    /// Customize annotation markers and cues while retaining the frame glyphs.
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
