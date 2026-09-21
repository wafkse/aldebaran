//! A module which hosts character set definitions that are used to shape an
//! abstract box.
//!
//! The primary item of interest in this module is the [`Delimit`] type.

/// A set of delimiting characters.
///
/// These characters are used to induce a spatial separation between different
/// parts of the error report.
///
/// This is given as the set of components that make up a box that is segmented
/// into four frames.
///
/// The enumeration of the components is as follows:
/// - `left`: The leftmost character of the box.
/// - `right`: The rightmost character of the box.
/// - `top`: The topmost character of the box.
/// - `bottom`: The bottommost character of the box.
/// - `join`: four-way intersection character between the *previous four* characters.
/// - `corner`: The four corner characters of the box. See [`Corner`] for more.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Delimit {
    /// The left delimiter character.
    ///
    /// This is usually the same as [`the right character`](Delimit::right).
    ///
    /// The character used for this specific purpose is usually a full-width
    /// vertical bar, usually centered.
    left: char,
    /// The right delimiter character.
    ///
    /// This is usually the same as [`the left character`](Delimit::left).
    ///
    /// The character used for this specific purpose is usually a full-width
    /// vertical bar, usually centered.
    right: char,
    /// A top delimiter character.
    ///
    /// This is usually the same as [`the left character`](Delimit::bottom).
    ///
    /// The character used for this specific purpose is usually a full-width
    /// horizontal bar, usually centered.
    top: char,
    /// A bottom delimiter character.
    ///
    /// This is usually the same as [`the top character`](Delimit::top).
    ///
    /// The character used for this specific purpose is usually a full-width
    /// horizontal bar, usually centered.
    bottom: char,
    /// The subset of joiner characters.
    join: Join,
    /// The subset of box corner characters.
    corner: Corner,
}

impl Delimit {
    /// The delimit characters of a box with **strictly** *ASCII* characters.
    ///
    /// This is also the [`default`](Default) value for [`Delimit`].
    pub const ASCII: Self = Self::tuple(('|', '|', '-', '-', Join::ASCII, Corner::ASCII));

    /// The delimit characters of a box with *unicode* characters.
    pub const UNICODE: Self = Self::tuple(('│', '│', '─', '─', Join::UNICODE, Corner::UNICODE));

    /// The standard delimit character set.
    ///
    /// Currently, this is set to [`ASCII`](Delimit::ASCII).
    #[inline]
    pub const fn standard() -> Self {
        Self::ASCII
    }
}

impl Delimit {
    /// Create a new [`Delimit`] from a tuple of each delimiter kind and a
    /// [`Corner`].
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

    /// Retrieve the *left* delimiter character for [`this set`](Delimit).
    #[inline]
    pub const fn left(&self) -> char {
        let &Self { left, .. } = self;

        left
    }

    /// Retrieve *(an immutable reference)* to the *left* delimiter character
    /// for [`this set`](Delimit).
    #[inline]
    pub const fn left_ref(&self) -> &char {
        let &Self { ref left, .. } = self;

        left
    }

    /// Retrieve *(a mutable reference)* to the *left* delimiter character for
    /// [`this set`](Delimit).
    #[inline]
    pub const fn left_mut(&mut self) -> &mut char {
        let &mut Self { ref mut left, .. } = self;

        left
    }

    /// Retrieve the *right* delimiter character for [`this set`](Delimit).
    #[inline]
    pub const fn right(&self) -> char {
        let &Self { right, .. } = self;

        right
    }

    /// Retrieve *(an immutable reference)* to the *right* delimiter character
    /// for [`this set`](Delimit).
    #[inline]
    pub const fn right_ref(&self) -> &char {
        let &Self { ref right, .. } = self;

        right
    }

    /// Retrieve *(a mutable reference)* to the *right* delimiter character for
    /// [`this set`](Delimit).
    #[inline]
    pub const fn right_mut(&mut self) -> &mut char {
        let &mut Self { ref mut right, .. } = self;

        right
    }

    /// Retrieve the *top* delimiter character for [`this set`](Delimit).
    #[inline]
    pub const fn top(&self) -> char {
        let &Self { top, .. } = self;

        top
    }

    /// Retrieve *(an immutable reference)* to the *top* delimiter character for
    /// [`this set`](Delimit).
    #[inline]
    pub const fn top_ref(&self) -> &char {
        let &Self { ref top, .. } = self;

        top
    }

    /// Retrieve *(a mutable reference)* to the *top* delimiter character for
    /// [`this set`](Delimit).
    #[inline]
    pub const fn top_mut(&mut self) -> &mut char {
        let &mut Self { ref mut top, .. } = self;

        top
    }

    /// Retrieve the *bottom* delimiter character for [`this set`](Delimit).
    #[inline]
    pub const fn bottom(&self) -> char {
        let &Self { bottom, .. } = self;

        bottom
    }

    /// Retrieve *(an immutable reference)* to the *bottom* delimiter character
    /// for [`this set`](Delimit).
    #[inline]
    pub const fn bottom_ref(&self) -> &char {
        let &Self { ref bottom, .. } = self;

        bottom
    }

    /// Retrieve *(a mutable reference)* to the *bottom* delimiter character for
    /// [`this set`](Delimit).
    #[inline]
    pub const fn bottom_mut(&mut self) -> &mut char {
        let &mut Self { ref mut bottom, .. } = self;

        bottom
    }

    /// Retrieve the *join* delimiter character for [`this set`](Delimit).
    #[inline]
    pub const fn join(&self) -> Join {
        let &Self { join, .. } = self;

        join
    }

    /// Retrieve *(an immutable reference)* to the *join* delimiter character
    /// for [`this set`](Delimit).
    #[inline]
    pub const fn join_ref(&self) -> &Join {
        let &Self { ref join, .. } = self;

        join
    }

    /// Retrieve *(a mutable reference)* to the *join* delimiter character for
    /// [`this set`](Delimit).
    #[inline]
    pub const fn join_mut(&mut self) -> &mut Join {
        let &mut Self { ref mut join, .. } = self;

        join
    }

    /// Retrieve the *corner* characters for [`this set`](Delimit).
    #[inline]
    pub const fn corner(&self) -> Corner {
        let &Self { corner, .. } = self;

        corner
    }

    /// Retrieve *(an immutable reference)* to the *corner* characters for
    /// [`this set`](Delimit).
    #[inline]
    pub const fn corner_ref(&self) -> &Corner {
        let &Self { ref corner, .. } = self;

        corner
    }

    /// Retrieve *(a mutable reference)* to the *corner* characters for
    /// [`this set`](Delimit).
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

/// A set of join characters.
///
/// This is used to extend the [`Delimit`] set to include characters that join
/// the different intersection points of the box.
///
/// This set serves a purely descriptive purpose, so, in the cases where the
/// character cannot be used, it is recommented to use the one that is closest
/// to the desired character. A key example of this is the use of the `+` symbol
/// when constrained to strictly *ASCII* characters.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Join {
    /// The character used to join the left, right, up, and down parts of the
    /// box.
    ///
    /// Example: `┼`
    four_way: char,
    /// The character used to join the left, right, and up parts of the box.
    ///
    /// Example: `┴`
    left_right_up: char,
    /// The character used to join the left, right, and down parts of the box.
    ///
    /// Example: `┬`
    left_right_down: char,
    /// The character used to join the left, up, and down parts of the box.
    ///
    /// Example:  `┤`
    left_up_down: char,
    /// The character used to join the right, up, and down parts of the box.
    ///
    /// Example: `├`
    right_up_down: char,
}

impl Default for Join {
    #[inline]
    fn default() -> Self {
        Self::standard()
    }
}

impl Join {
    /// The join characters of a box with **strictly** *ASCII* characters.
    ///
    /// This is also the [`default`](Default) value for [`Join`].
    pub const ASCII: Self = Self::tuple(('+', '+', '+', '+', '+'));

    /// The join characters of a box with *unicode* characters.
    pub const UNICODE: Self = Self::tuple(('┼', '┴', '┬', '┤', '├'));

    /// The standard join character set.
    ///
    /// Currently, this is set to [`ASCII`](Join::ASCII).
    #[inline]
    pub const fn standard() -> Self {
        Self::ASCII
    }
}

impl Join {
    /// Create a new [`Join`] from its raw parts: each joiner kind.
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

    /// Create a new [`Join`] from a 5-element tuple of each joiner kind.
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

    /// Create a new [`Join`] from a 5-element array of each joiner kind.
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

    /// Retrieve the *four-way* join character for [`this set`](Join).
    #[inline]
    pub const fn four_way(&self) -> char {
        let &Self { four_way, .. } = self;

        four_way
    }

    /// Retrieve *(an immutable reference)* to the *four-way* join character for
    /// [`this set`](Join).
    #[inline]
    pub const fn four_way_ref(&self) -> &char {
        let &Self { ref four_way, .. } = self;

        four_way
    }

    /// Retrieve *(a mutable reference)* to the *four-way* join character for
    /// [`this set`](Join).
    #[inline]
    pub const fn four_way_mut(&mut self) -> &mut char {
        let &mut Self { ref mut four_way, .. } = self;

        four_way
    }

    /// Retrieve the *left-right-up* join character for [`this set`](Join).
    #[inline]
    pub const fn left_right_up(&self) -> char {
        let &Self { left_right_up, .. } = self;

        left_right_up
    }

    /// Retrieve *(an immutable reference)* to the *left-right-up* join
    /// character for [`this set`](Join).
    #[inline]
    pub const fn left_right_up_ref(&self) -> &char {
        let &Self { ref left_right_up, .. } = self;

        left_right_up
    }

    /// Retrieve *(a mutable reference)* to the *left-right-up* join character
    /// for [`this set`](Join).
    #[inline]
    pub const fn left_right_up_mut(&mut self) -> &mut char {
        let &mut Self { ref mut left_right_up, .. } = self;

        left_right_up
    }

    /// Retrieve the *left-right-down* join character for [`this set`](Join).
    #[inline]
    pub const fn left_right_down(&self) -> char {
        let &Self { left_right_down, .. } = self;

        left_right_down
    }

    /// Retrieve *(an immutable reference)* to the *left-right-down* join
    /// character for [`this set`](Join).
    #[inline]
    pub const fn left_right_down_ref(&self) -> &char {
        let &Self { ref left_right_down, .. } = self;

        left_right_down
    }

    /// Retrieve *(a mutable reference)* to the *left-right-down* join character
    /// for [`this set`](Join).
    #[inline]
    pub const fn left_right_down_mut(&mut self) -> &mut char {
        let &mut Self {
            ref mut left_right_down, ..
        } = self;

        left_right_down
    }

    /// Retrieve the *left-up-down* join character for [`this set`](Join).
    #[inline]
    pub const fn left_up_down(&self) -> char {
        let &Self { left_up_down, .. } = self;

        left_up_down
    }

    /// Retrieve *(an immutable reference)* to the *left-up-down* join character
    /// for [`this set`](Join).
    #[inline]
    pub const fn left_up_down_ref(&self) -> &char {
        let &Self { ref left_up_down, .. } = self;

        left_up_down
    }

    /// Retrieve *(a mutable reference)* to the *left-up-down* join character
    /// for [`this set`](Join).
    #[inline]
    pub const fn left_up_down_mut(&mut self) -> &mut char {
        let &mut Self { ref mut left_up_down, .. } = self;

        left_up_down
    }

    /// Retrieve the *right-up-down* join character for [`this set`](Join).
    #[inline]
    pub const fn right_up_down(&self) -> char {
        let &Self { right_up_down, .. } = self;

        right_up_down
    }

    /// Retrieve *(an immutable reference)* to the *right-up-down* join
    /// character for [`this set`](Join).
    #[inline]
    pub const fn right_up_down_ref(&self) -> &char {
        let &Self { ref right_up_down, .. } = self;

        right_up_down
    }

    /// Retrieve *(a mutable reference)* to the *right-up-down* join character
    /// for [`this set`](Join).
    #[inline]
    pub const fn right_up_down_mut(&mut self) -> &mut char {
        let &mut Self { ref mut right_up_down, .. } = self;

        right_up_down
    }
}

/// The four corners of a box.
///
/// See [`Delimit`] for further information.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Corner {
    top_left: char,
    top_right: char,
    bottom_left: char,
    bottom_right: char,
}

impl Default for Corner {
    #[inline]
    fn default() -> Self {
        Self::standard()
    }
}

impl Corner {
    /// The corners of a box with **strictly** *ASCII* characters.
    ///
    /// This is also the [`default`](Default) value for [`Corner`].
    ///
    /// Note that this does not make use of the
    pub const ASCII: Self = Self::tuple(('+', '+', '+', '+'));

    /// The corners of a box with *unicode* characters.
    pub const UNICODE: Self = Self::tuple(('┌', '┐', '└', '┘'));

    /// The standard corner character set.
    ///
    /// Currently, this is set to [`ASCII`](Corner::ASCII).
    #[inline]
    pub const fn standard() -> Self {
        Self::ASCII
    }
}

impl Corner {
    /// Create a new [`Corner`] from its raw parts: each corner kind.
    #[inline]
    pub const fn from_raw_parts(top_left: char, top_right: char, bottom_left: char, bottom_right: char) -> Self {
        Self {
            top_left,
            top_right,
            bottom_left,
            bottom_right,
        }
    }

    /// Create a new [`Corner`] from a 4-element tuple of each corner kind.
    #[inline]
    pub const fn tuple((top_left, top_right, bottom_left, bottom_right): (char, char, char, char)) -> Self {
        Self {
            top_left,
            top_right,
            bottom_left,
            bottom_right,
        }
    }

    /// Create a new [`Corner`] from a 4-elenent array of each corner kind.
    #[inline]
    pub const fn array([top_left, top_right, bottom_left, bottom_right]: [char; 4]) -> Self {
        Self {
            top_left,
            top_right,
            bottom_left,
            bottom_right,
        }
    }

    /// Retrieve the *top-left* corner character for [`this set`](Corner).
    #[inline]
    pub const fn top_left(&self) -> char {
        let &Self { top_left, .. } = self;

        top_left
    }

    /// Retrieve *(an immutable reference)* to the *top-left* corner character
    /// for [`this set`](Corner).
    #[inline]
    pub const fn top_left_ref(&self) -> &char {
        let &Self { ref top_left, .. } = self;

        top_left
    }

    /// Retrieve *(a mutable reference)* to the *top-left* corner character for
    /// [`this set`](Corner).
    #[inline]
    pub const fn top_left_mut(&mut self) -> &mut char {
        let &mut Self { ref mut top_left, .. } = self;

        top_left
    }

    /// Retrieve the *top-right* corner character for [`this set`](Corner).
    #[inline]
    pub const fn top_right(&self) -> char {
        let &Self { top_right, .. } = self;

        top_right
    }

    /// Retrieve *(an immutable reference)* to the *top-right* corner character
    /// for [`this set`](Corner).
    #[inline]
    pub const fn top_right_ref(&self) -> &char {
        let &Self { ref top_right, .. } = self;

        top_right
    }

    /// Retrieve *(a mutable reference)* to the *top-right* corner character for
    /// [`this set`](Corner).
    #[inline]
    pub const fn top_right_mut(&mut self) -> &mut char {
        let &mut Self { ref mut top_right, .. } = self;

        top_right
    }

    /// Retrieve the *bottom-left* corner character for [`this set`](Corner).
    #[inline]
    pub const fn bottom_left(&self) -> char {
        let &Self { bottom_left, .. } = self;

        bottom_left
    }

    /// Retrieve *(an immutable reference)* to the *bottom-left* corner
    /// character for [`this set`](Corner).
    #[inline]
    pub const fn bottom_left_ref(&self) -> &char {
        let &Self { ref bottom_left, .. } = self;

        bottom_left
    }

    /// Retrieve *(a mutable reference)* to the *bottom-left* corner character
    /// for [`this set`](Corner).
    #[inline]
    pub const fn bottom_left_mut(&mut self) -> &mut char {
        let &mut Self { ref mut bottom_left, .. } = self;

        bottom_left
    }

    /// Retrieve the *bottom-right* corner character for [`this set`](Corner).
    #[inline]
    pub const fn bottom_right(&self) -> char {
        let &Self { bottom_right, .. } = self;

        bottom_right
    }

    /// Retrieve *(an immutable reference)* to the *bottom-right* corner
    /// character for [`this set`](Corner).
    #[inline]
    pub const fn bottom_right_ref(&self) -> &char {
        let &Self { ref bottom_right, .. } = self;

        bottom_right
    }

    /// Retrieve *(a mutable reference)* to the *bottom-right* corner character
    /// for [`this set`](Corner).
    #[inline]
    pub const fn bottom_right_mut(&mut self) -> &mut char {
        let &mut Self { ref mut bottom_right, .. } = self;

        bottom_right
    }
}
