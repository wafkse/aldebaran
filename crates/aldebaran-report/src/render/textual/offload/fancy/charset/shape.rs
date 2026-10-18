//! Edge, intersection, and corner symbols for source excerpt framing.
//!
//! [`Delimit`] combines straight edges with [`Join`] intersections and [`Corner`]
//! turns. [`Fancy`](super::super::Fancy) uses the left edge as its gutter
//! separator, the top edge and top-left corner at the source heading, and the
//! bottom edge and bottom-right corner at the closing gutter. The other frame
//! symbols remain configurable without being used by the current renderer.

/// Straight edges and their connecting symbols for framing source excerpts.
///
/// Edge glyphs can vary independently from the intersections in [`Join`] and
/// turns in [`Corner`]. Fancy uses a subset of these symbols to frame the gutter
/// and heading rather than enclosing every source row.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
// NOTE(invariant): None.
pub struct Delimit {
    /// Vertical separator between the line-number gutter and source content.
    left: char,

    /// Right boundary symbol, unused by the current Fancy renderer.
    right: char,

    /// Horizontal stroke between the opening corner and source heading.
    top: char,

    /// Horizontal stroke repeated across the gutter at the excerpt end.
    bottom: char,

    /// Symbols for crossings and branches, separate from corner turns.
    join: Join,

    /// Turns connecting horizontal and vertical edges at frame corners.
    corner: Corner,
}

impl Delimit {
    /// ASCII glyphs for frame edges, intersections, and corners.
    pub const ASCII: Self = Self::tuple(('|', '|', '-', '-', Join::ASCII, Corner::ASCII));

    /// Unicode glyphs for frame edges, intersections, and corners.
    pub const UNICODE: Self = Self::tuple(('│', '│', '─', '─', Join::UNICODE, Corner::UNICODE));

    /// ASCII edges and `+` connections used by [`Default`].
    #[inline]
    pub const fn standard() -> Self {
        Self::ASCII
    }
}

impl Delimit {
    /// Order frame parts as left, right, top, bottom, intersections, and corners.
    #[inline]
    pub const fn tuple((left, right, top, bottom, join, corner): (char, char, char, char, Join, Corner)) -> Self {
        Self {
            left,
            right,
            top,
            bottom,
            join,
            corner,
        }
    }

    /// Separator repeated beside source, marker, and annotation-message rows.
    #[inline]
    pub const fn left(&self) -> char {
        let &Self { left, .. } = self;

        left
    }

    /// Vertical gutter separator, `|` in ASCII and `│` in Unicode.
    #[inline]
    pub const fn left_ref(&self) -> &char {
        let &Self { ref left, .. } = self;

        left
    }

    /// Customize the separator that aligns source and annotation rows.
    #[inline]
    pub const fn left_mut(&mut self) -> &mut char {
        let &mut Self { ref mut left, .. } = self;

        left
    }

    /// Right boundary symbol available for framing beyond the gutter.
    #[inline]
    pub const fn right(&self) -> char {
        let &Self { right, .. } = self;

        right
    }

    /// Configured right edge, unused by the current Fancy renderer.
    #[inline]
    pub const fn right_ref(&self) -> &char {
        let &Self { ref right, .. } = self;

        right
    }

    /// Customize the right boundary independently of the gutter separator.
    #[inline]
    pub const fn right_mut(&mut self) -> &mut char {
        let &mut Self { ref mut right, .. } = self;

        right
    }

    /// Horizontal stroke following the opening corner before the source heading.
    #[inline]
    pub const fn top(&self) -> char {
        let &Self { top, .. } = self;

        top
    }

    /// Opening horizontal stroke, `-` in ASCII and `─` in Unicode.
    #[inline]
    pub const fn top_ref(&self) -> &char {
        let &Self { ref top, .. } = self;

        top
    }

    /// Customize the stroke connecting the opening corner to source metadata.
    #[inline]
    pub const fn top_mut(&mut self) -> &mut char {
        let &mut Self { ref mut top, .. } = self;

        top
    }

    /// Stroke repeated below the line-number gutter at the excerpt end.
    #[inline]
    pub const fn bottom(&self) -> char {
        let &Self { bottom, .. } = self;

        bottom
    }

    /// Closing horizontal stroke, configured independently of the heading stroke.
    #[inline]
    pub const fn bottom_ref(&self) -> &char {
        let &Self { ref bottom, .. } = self;

        bottom
    }

    /// Customize the line that closes the gutter beneath the excerpt.
    #[inline]
    pub const fn bottom_mut(&mut self) -> &mut char {
        let &mut Self { ref mut bottom, .. } = self;

        bottom
    }

    /// Crossing and branch symbols for connections involving at least three directions.
    #[inline]
    pub const fn join(&self) -> Join {
        let &Self { join, .. } = self;

        join
    }

    /// Intersection configuration, unused by the current Fancy renderer.
    #[inline]
    pub const fn join_ref(&self) -> &Join {
        let &Self { ref join, .. } = self;

        join
    }

    /// Customize crossings and branches without changing edges or corners.
    #[inline]
    pub const fn join_mut(&mut self) -> &mut Join {
        let &mut Self { ref mut join, .. } = self;

        join
    }

    /// Turns that connect pairs of perpendicular frame edges.
    #[inline]
    pub const fn corner(&self) -> Corner {
        let &Self { corner, .. } = self;

        corner
    }

    /// Corner configuration containing the opening and closing gutter turns.
    #[inline]
    pub const fn corner_ref(&self) -> &Corner {
        let &Self { ref corner, .. } = self;

        corner
    }

    /// Customize heading and closing turns independently of straight edges.
    #[inline]
    pub const fn corner_mut(&mut self) -> &mut Corner {
        let &mut Self { ref mut corner, .. } = self;

        corner
    }
}

impl Default for Delimit {
    #[inline]
    fn default() -> Self {
        Self::standard()
    }
}

/// Symbols for crossings and branches between frame edges.
///
/// Names identify the directions of strokes leaving the intersection. The
/// Unicode set distinguishes those connections, while ASCII uses `+` for all
/// of them.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
// NOTE(invariant): None.
pub struct Join {
    /// Crossing with strokes in all four directions.
    four_way: char,

    /// Horizontal branch whose third stroke extends upward.
    left_right_up: char,

    /// Horizontal branch whose third stroke extends downward.
    left_right_down: char,

    /// Vertical branch whose third stroke extends leftward.
    left_up_down: char,

    /// Vertical branch whose third stroke extends rightward.
    right_up_down: char,
}

impl Default for Join {
    #[inline]
    fn default() -> Self {
        Self::standard()
    }
}

impl Join {
    /// ASCII glyphs for frame intersections.
    pub const ASCII: Self = Self::tuple(('+', '+', '+', '+', '+'));

    /// Unicode glyphs for frame intersections.
    pub const UNICODE: Self = Self::tuple(('┼', '┴', '┬', '┤', '├'));

    /// Use `+` for every crossing and branch, as [`Default`] does.
    #[inline]
    pub const fn standard() -> Self {
        Self::ASCII
    }
}

impl Join {
    /// Configure a crossing and four branches by their outgoing stroke directions.
    #[inline]
    pub const fn from_raw_parts(
        four_way: char,
        left_right_up: char,
        left_right_down: char,
        left_up_down: char,
        right_up_down: char,
    ) -> Self {
        Self {
            four_way,
            left_right_up,
            left_right_down,
            left_up_down,
            right_up_down,
        }
    }

    /// Order connections as four-way, left-right-up, left-right-down, left-up-down, and right-up-down.
    #[inline]
    pub const fn tuple((four_way, left_right_up, left_right_down, left_up_down, right_up_down): (char, char, char, char, char)) -> Self {
        Self {
            four_way,
            left_right_up,
            left_right_down,
            left_up_down,
            right_up_down,
        }
    }

    /// Use the same crossing and branch order as [`Self::tuple`].
    #[inline]
    pub const fn array([four_way, left_right_up, left_right_down, left_up_down, right_up_down]: [char; 5]) -> Self {
        Self {
            four_way,
            left_right_up,
            left_right_down,
            left_up_down,
            right_up_down,
        }
    }

    /// Crossing with horizontal and vertical strokes, `┼` in Unicode.
    #[inline]
    pub const fn four_way(&self) -> char {
        let &Self { four_way, .. } = self;

        four_way
    }

    /// Interior crossing that connects all four neighboring edge segments.
    #[inline]
    pub const fn four_way_ref(&self) -> &char {
        let &Self { ref four_way, .. } = self;

        four_way
    }

    /// Customize the crossing independently of three-direction branches.
    #[inline]
    pub const fn four_way_mut(&mut self) -> &mut char {
        let &mut Self { ref mut four_way, .. } = self;

        four_way
    }

    /// Horizontal connection with an upward branch, `┴` in Unicode.
    #[inline]
    pub const fn left_right_up(&self) -> char {
        let &Self { left_right_up, .. } = self;

        left_right_up
    }

    /// Branch open to the left, right, and above, with no downward stroke.
    #[inline]
    pub const fn left_right_up_ref(&self) -> &char {
        let &Self { ref left_right_up, .. } = self;

        left_right_up
    }

    /// Customize the upward branch of a horizontal frame edge.
    #[inline]
    pub const fn left_right_up_mut(&mut self) -> &mut char {
        let &mut Self { ref mut left_right_up, .. } = self;

        left_right_up
    }

    /// Horizontal connection with a downward branch, `┬` in Unicode.
    #[inline]
    pub const fn left_right_down(&self) -> char {
        let &Self { left_right_down, .. } = self;

        left_right_down
    }

    /// Branch open to the left, right, and below, with no upward stroke.
    #[inline]
    pub const fn left_right_down_ref(&self) -> &char {
        let &Self { ref left_right_down, .. } = self;

        left_right_down
    }

    /// Customize the downward branch of a horizontal frame edge.
    #[inline]
    pub const fn left_right_down_mut(&mut self) -> &mut char {
        let &mut Self {
            ref mut left_right_down, ..
        } = self;

        left_right_down
    }

    /// Vertical connection with a leftward branch, `┤` in Unicode.
    #[inline]
    pub const fn left_up_down(&self) -> char {
        let &Self { left_up_down, .. } = self;

        left_up_down
    }

    /// Branch open above, below, and to the left, with no rightward stroke.
    #[inline]
    pub const fn left_up_down_ref(&self) -> &char {
        let &Self { ref left_up_down, .. } = self;

        left_up_down
    }

    /// Customize the leftward branch of a vertical frame edge.
    #[inline]
    pub const fn left_up_down_mut(&mut self) -> &mut char {
        let &mut Self { ref mut left_up_down, .. } = self;

        left_up_down
    }

    /// Vertical connection with a rightward branch, `├` in Unicode.
    #[inline]
    pub const fn right_up_down(&self) -> char {
        let &Self { right_up_down, .. } = self;

        right_up_down
    }

    /// Branch open above, below, and to the right, with no leftward stroke.
    #[inline]
    pub const fn right_up_down_ref(&self) -> &char {
        let &Self { ref right_up_down, .. } = self;

        right_up_down
    }

    /// Customize the rightward branch of a vertical frame edge.
    #[inline]
    pub const fn right_up_down_mut(&mut self) -> &mut char {
        let &mut Self { ref mut right_up_down, .. } = self;

        right_up_down
    }
}

/// Turns connecting perpendicular edges of a [`Delimit`] frame.
///
/// Corner names describe positions in a surrounding frame. Their strokes face
/// inward, so the top-left corner connects rightward and downward, while the
/// bottom-right connects leftward and upward.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
// NOTE(invariant): None.
pub struct Corner {
    /// Rightward and downward turn used at the source heading.
    top_left: char,

    /// Leftward and downward turn at a frame's upper-right boundary.
    top_right: char,

    /// Rightward and upward turn at a frame's lower-left boundary.
    bottom_left: char,

    /// Leftward and upward turn used to close the gutter.
    bottom_right: char,
}

impl Default for Corner {
    #[inline]
    fn default() -> Self {
        Self::standard()
    }
}

impl Corner {
    /// ASCII glyphs for frame corners.
    pub const ASCII: Self = Self::tuple(('+', '+', '+', '+'));

    /// Unicode glyphs for frame corners.
    pub const UNICODE: Self = Self::tuple(('┌', '┐', '└', '┘'));

    /// Use `+` for every frame turn, as [`Default`] does.
    #[inline]
    pub const fn standard() -> Self {
        Self::ASCII
    }
}

impl Corner {
    /// Configure turns for the top-left, top-right, bottom-left, and bottom-right positions.
    #[inline]
    pub const fn from_raw_parts(top_left: char, top_right: char, bottom_left: char, bottom_right: char) -> Self {
        Self {
            top_left,
            top_right,
            bottom_left,
            bottom_right,
        }
    }

    /// Order turns as top-left, top-right, bottom-left, and bottom-right.
    #[inline]
    pub const fn tuple((top_left, top_right, bottom_left, bottom_right): (char, char, char, char)) -> Self {
        Self {
            top_left,
            top_right,
            bottom_left,
            bottom_right,
        }
    }

    /// Use top-left, top-right, bottom-left, and bottom-right turns, matching [`Self::tuple`].
    #[inline]
    pub const fn array([top_left, top_right, bottom_left, bottom_right]: [char; 4]) -> Self {
        Self {
            top_left,
            top_right,
            bottom_left,
            bottom_right,
        }
    }

    /// Opening turn from the gutter separator toward the source heading.
    #[inline]
    pub const fn top_left(&self) -> char {
        let &Self { top_left, .. } = self;

        top_left
    }

    /// Rightward and downward corner, `┌` in the Unicode preset.
    #[inline]
    pub const fn top_left_ref(&self) -> &char {
        let &Self { ref top_left, .. } = self;

        top_left
    }

    /// Customize the turn at the start of the source heading.
    #[inline]
    pub const fn top_left_mut(&mut self) -> &mut char {
        let &mut Self { ref mut top_left, .. } = self;

        top_left
    }

    /// Upper-right turn connecting leftward and downward strokes.
    #[inline]
    pub const fn top_right(&self) -> char {
        let &Self { top_right, .. } = self;

        top_right
    }

    /// Leftward and downward corner, `┐` in the Unicode preset.
    #[inline]
    pub const fn top_right_ref(&self) -> &char {
        let &Self { ref top_right, .. } = self;

        top_right
    }

    /// Customize the upper-right turn independently of the heading corner.
    #[inline]
    pub const fn top_right_mut(&mut self) -> &mut char {
        let &mut Self { ref mut top_right, .. } = self;

        top_right
    }

    /// Lower-left turn connecting rightward and upward strokes.
    #[inline]
    pub const fn bottom_left(&self) -> char {
        let &Self { bottom_left, .. } = self;

        bottom_left
    }

    /// Rightward and upward corner, `└` in the Unicode preset.
    #[inline]
    pub const fn bottom_left_ref(&self) -> &char {
        let &Self { ref bottom_left, .. } = self;

        bottom_left
    }

    /// Customize the lower-left turn independently of the closing gutter corner.
    #[inline]
    pub const fn bottom_left_mut(&mut self) -> &mut char {
        let &mut Self { ref mut bottom_left, .. } = self;

        bottom_left
    }

    /// Closing turn from the gutter separator toward the bottom stroke.
    #[inline]
    pub const fn bottom_right(&self) -> char {
        let &Self { bottom_right, .. } = self;

        bottom_right
    }

    /// Leftward and upward corner, `┘` in the Unicode preset.
    #[inline]
    pub const fn bottom_right_ref(&self) -> &char {
        let &Self { ref bottom_right, .. } = self;

        bottom_right
    }

    /// Customize the turn at the end of the closing gutter stroke.
    #[inline]
    pub const fn bottom_right_mut(&mut self) -> &mut char {
        let &mut Self { ref mut bottom_right, .. } = self;

        bottom_right
    }
}
