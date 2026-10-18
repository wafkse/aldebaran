//! A module which hosts character set definitions that are used consistently
//! convey an indication of some sort.
//!
//! The primary item of interest in this module is the [`Indicator`] type.

/// A set of indicator characters.
///
/// An indicator character is used to emphasize the importance of a particular
/// part of the error report, or to indicate a particular relationship between
/// different parts of the error report.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Indicator {
    /// The subset of arrow characters to be used as spatial indicators.
    arrow: Arrow,

    /// The subset of point characters to be used as spatial indicators.
    point: Point,

    /// The subset of characters to be used as connotative indicators.
    connote: Connote,

    /// The subset of characters to be used as distictive markers.
    ///
    /// This is used to mark a particular region of the source code, as
    /// part of the rendered error report.
    marker: Marker<'static>,
}

impl Indicator {
    /// A set of indicator characters that are strictly *ASCII*.
    pub const ASCII: Self = Self::tuple((Arrow::ASCII, Point::ASCII, Connote::ASCII, Marker::ASCII));

    /// A set of indicator characters that are strictly *unicode*.
    pub const UNICODE: Self = Self::tuple((Arrow::UNICODE, Point::UNICODE, Connote::UNICODE, Marker::UNICODE));

    /// The standard indicator character set.
    ///
    /// Currently, this is set to [`ASCII`](Indicator::ASCII).
    #[inline]
    pub const fn standard() -> Self {
        Self::ASCII
    }
}

impl Indicator {
    /// Create a new [`Indicator`] from its raw parts.
    #[inline]
    pub const fn from_raw_parts(arrow: Arrow, point: Point, connote: Connote, marker: Marker<'static>) -> Self {
        Self {
            arrow,
            point,
            connote,
            marker,
        }
    }

    /// Create a new [`Indicator`] from a tuple of its parts.
    #[inline]
    pub const fn tuple((arrow, point, connote, marker): (Arrow, Point, Connote, Marker<'static>)) -> Self {
        Self {
            arrow,
            point,
            connote,
            marker,
        }
    }

    /// Retrieve the arrow characters for [`this set`](Indicator).
    #[inline]
    pub const fn arrow(&self) -> Arrow {
        let &Self { arrow, .. } = self;

        arrow
    }

    /// Retrieve *(an immutable reference)* to the arrow characters for [`this
    /// set`](Indicator).
    #[inline]
    pub const fn arrow_ref(&self) -> &Arrow {
        let &Self { ref arrow, .. } = self;

        arrow
    }

    /// Retrieve *(a mutable reference)* to the arrow characters for [`this
    /// set`](Indicator).
    #[inline]
    pub const fn arrow_mut(&mut self) -> &mut Arrow {
        let &mut Self { ref mut arrow, .. } = self;

        arrow
    }

    /// Retrieve the point characters for [`this set`](Indicator).
    #[inline]
    pub const fn point(&self) -> Point {
        let &Self { point, .. } = self;

        point
    }

    /// Retrieve *(an immutable reference)* to the point characters for [`this
    /// set`](Indicator).
    #[inline]
    pub const fn point_ref(&self) -> &Point {
        let &Self { ref point, .. } = self;

        point
    }

    /// Retrieve *(a mutable reference)* to the point characters for [`this
    /// set`](Indicator).
    #[inline]
    pub const fn point_mut(&mut self) -> &mut Point {
        let &mut Self { ref mut point, .. } = self;

        point
    }

    /// Retrieve the connotative characters for [`this set`](Indicator).
    #[inline]
    pub const fn connote(&self) -> Connote {
        let &Self { connote, .. } = self;

        connote
    }

    /// Retrieve *(an immutable reference)* to the connotative characters for
    /// [`this set`](Indicator).
    #[inline]
    pub const fn connote_ref(&self) -> &Connote {
        let &Self { ref connote, .. } = self;

        connote
    }

    /// Retrieve *(a mutable reference)* to the connotative characters for
    /// [`this set`](Indicator).
    #[inline]
    pub const fn connote_mut(&mut self) -> &mut Connote {
        let &mut Self { ref mut connote, .. } = self;

        connote
    }

    /// Retrieve the distinctive markers for [`this set`](Indicator).
    #[inline]
    pub const fn marker(&self) -> Marker<'static> {
        let &Self { marker, .. } = self;

        marker
    }

    /// Retrieve *(an immutable reference)* to the distinctive markers for
    /// [`this set`](Indicator).
    #[inline]
    pub const fn marker_ref(&self) -> &Marker<'static> {
        let &Self { ref marker, .. } = self;

        marker
    }

    /// Retrieve *(a mutable reference)* to the distinctive markers for
    /// [`this set`](Indicator).
    #[inline]
    pub const fn marker_mut(&mut self) -> &mut Marker<'static> {
        let &mut Self { ref mut marker, .. } = self;

        marker
    }
}

impl Default for Indicator {
    #[inline]
    fn default() -> Self {
        Self::standard()
    }
}

/// A set of connotative characters.
///
/// Albeit most of the time these characters are completely arbitrary and
/// optional, they are used to convey a particular *connotation* in the context
/// of the error report. For example, a [`Connote::warning`] character is used
/// to introduce a warning label and to provide a visual cue to the reader that
/// the error is not necessarily an error, but a warning.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Connote {
    /// The character used to indicate a *warning* or otherwise *cautionary*
    /// connotation.
    warning: char,
    /// The character used to indicate an *error* or otherwise *punitive*
    /// connotation.
    error: char,
    /// The character used to indicate the existence of *a note*.
    ///
    /// Albeit this definition is quite broad, it is used to indicate a part of
    /// the error report that is not necessarily relevant to the error
    /// itself, but to the context of the error, and must be taken with a grain
    /// of salt.
    note: Option<char>,
    /// The character used to indicate *a helpful* connotation.
    help: Option<char>,
}

impl Connote {
    /// A set of *ASCII* connotative characters.
    pub const ASCII: Self = Self::tuple(('!', 'x', 'i', 'v'));

    /// A set of *unicode* connotative characters.
    pub const UNICODE: Self = Self::tuple(('◈', '❱', '⟐', '⟢'));

    /// The standard connotative character set.
    ///
    /// Currently, this is same as [`ASCII`](Connote::ASCII).
    #[inline]
    pub const fn standard() -> Self {
        Self::ASCII
    }
}

impl Connote {
    /// Create a new [`Connote`] from its raw parts: each connotative kind.
    #[inline]
    pub const fn from_raw_parts(warning: char, error: char, note: Option<char>, help: Option<char>) -> Self {
        Self {
            warning,
            error,
            note,
            help,
        }
    }

    /// Create a new [`Connote`] from a 4-element tuple of each connotative
    /// kind.
    #[inline]
    pub const fn tuple((warning, error, note, help): (char, char, char, char)) -> Self {
        let note = Some(note);
        let help = Some(help);

        Self {
            warning,
            error,
            note,
            help,
        }
    }

    /// Create a new [`Connote`] from a 4-element array of each connotative
    /// kind.
    #[inline]
    pub const fn array([warning, error, note, help]: [char; 4]) -> Self {
        let note = Some(note);
        let help = Some(help);

        Self {
            warning,
            error,
            note,
            help,
        }
    }

    /// Retrieve the *warning* connotative character for [`this set`](Connote).
    #[inline]
    pub const fn warning(&self) -> char {
        let &Self { warning, .. } = self;

        warning
    }

    /// Retrieve *(an immutable reference)* to the *warning* connotative
    /// character for [`this set`](Connote).
    #[inline]
    pub const fn warning_ref(&self) -> &char {
        let &Self { ref warning, .. } = self;

        warning
    }

    /// Retrieve *(a mutable reference)* to the *warning* connotative character
    /// for [`this set`](Connote).
    #[inline]
    pub const fn warning_mut(&mut self) -> &mut char {
        let &mut Self { ref mut warning, .. } = self;

        warning
    }

    /// Retrieve the *error* connotative character for [`this set`](Connote).
    #[inline]
    pub const fn error(&self) -> char {
        let &Self { error, .. } = self;

        error
    }

    /// Retrieve *(an immutable reference)* to the *error* connotative character
    /// for [`this set`](Connote).
    #[inline]
    pub const fn error_ref(&self) -> &char {
        let &Self { ref error, .. } = self;

        error
    }

    /// Retrieve *(a mutable reference)* to the *error* connotative character
    /// for [`this set`](Connote).
    #[inline]
    pub const fn error_mut(&mut self) -> &mut char {
        let &mut Self { ref mut error, .. } = self;

        error
    }

    /// Retrieve the *note* connotative character for [`this set`](Connote).
    #[inline]
    pub const fn note(&self) -> Option<char> {
        let &Self { note, .. } = self;

        note
    }

    /// Retrieve *(an immutable reference)* to the *note* connotative character
    /// for [`this set`](Connote).
    #[inline]
    pub const fn note_ref(&self) -> Option<&char> {
        let &Self { ref note, .. } = self;

        note.as_ref()
    }

    /// Retrieve *(a mutable reference)* to the *note* connotative character for
    /// [`this set`](Connote).
    #[inline]
    pub const fn note_mut(&mut self) -> Option<&mut char> {
        let &mut Self { ref mut note, .. } = self;

        note.as_mut()
    }

    /// Retrieve the *help* connotative character for [`this set`](Connote).
    #[inline]
    pub const fn help(&self) -> Option<char> {
        let &Self { help, .. } = self;

        help
    }

    /// Retrieve *(an immutable reference)* to the *help* connotative character
    /// for [`this set`](Connote).
    #[inline]
    pub const fn help_ref(&self) -> Option<&char> {
        let &Self { ref help, .. } = self;

        help.as_ref()
    }

    /// Retrieve *(a mutable reference)* to the *help* connotative character for
    /// [`this set`](Connote).
    #[inline]
    pub const fn help_mut(&mut self) -> Option<&mut char> {
        let &mut Self { ref mut help, .. } = self;

        help.as_mut()
    }
}

impl Default for Connote {
    #[inline]
    fn default() -> Self {
        Self::standard()
    }
}

/// A set of point characters.
///
/// Unlike the [`arrow characters`](Arrow), these characters are used to convey
/// a more robust relationship between different parts of the error report.
///
/// For example, pointers are used to finalize a long arrow, or to strongly
/// indicate a particular part of the error report.
///
/// In other words, this set assumes that the characters are going to be part of
/// a bigger scheme, unlike the arrow characters, which can be completely
/// free-standing.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Point {
    /// An upward-pointing character.
    up: char,
    /// A downward-pointing character.
    down: char,
    /// A leftward-pointing character.
    left: char,
    /// A rightward-pointing character.
    right: char,
}

impl Point {
    /// A set of *ASCII* point characters.
    pub const ASCII: Self = Self::tuple(('^', 'v', '<', '>'));

    /// A set of *unicode* point characters.
    pub const UNICODE: Self = Self::tuple(('▲', '▼', '◀', '▶'));

    /// The standard point character set.
    ///
    /// Currently, this is set to [`ASCII`](Point::ASCII).
    #[inline]
    pub const fn standard() -> Self {
        Self::ASCII
    }
}

impl Point {
    /// Create a new [`Point`] from its raw parts: each point kind (direction).
    #[inline]
    pub const fn from_raw_parts(up: char, down: char, left: char, right: char) -> Self {
        Self { up, down, left, right }
    }

    /// Create a new [`Point`] from a 4-element tuple of each point kind
    /// (direction).
    #[inline]
    pub const fn tuple((up, down, left, right): (char, char, char, char)) -> Self {
        Self { up, down, left, right }
    }

    /// Create a new [`Point`] from a 4-element array of each point kind
    /// (direction).
    #[inline]
    pub const fn array([up, down, left, right]: [char; 4]) -> Self {
        Self { up, down, left, right }
    }

    /// Retrieve the *upwards* point character for [`this set`](Point).
    #[inline]
    pub const fn up(&self) -> char {
        let &Self { up, .. } = self;

        up
    }

    /// Retrieve *(an immutable reference)* to the *upwards* point character for
    /// [`this set`](Point).
    #[inline]
    pub const fn up_ref(&self) -> &char {
        let &Self { ref up, .. } = self;

        up
    }

    /// Retrieve *(a mutable reference)* to the *upwards* point character for
    /// [`this set`](Point).
    #[inline]
    pub const fn up_mut(&mut self) -> &mut char {
        let &mut Self { ref mut up, .. } = self;

        up
    }

    /// Retrieve the *downwards* point character for [`this set`](Point).
    #[inline]
    pub const fn down(&self) -> char {
        let &Self { down, .. } = self;

        down
    }

    /// Retrieve *(an immutable reference)* to the *downwards* point character
    /// for [`this set`](Point).
    #[inline]
    pub const fn down_ref(&self) -> &char {
        let &Self { ref down, .. } = self;

        down
    }

    /// Retrieve *(a mutable reference)* to the *downwards* point character for
    /// [`this set`](Point).
    #[inline]
    pub const fn down_mut(&mut self) -> &mut char {
        let &mut Self { ref mut down, .. } = self;

        down
    }

    /// Retrieve the *leftwards* point character for [`this set`](Point).
    #[inline]
    pub const fn left(&self) -> char {
        let &Self { left, .. } = self;

        left
    }

    /// Retrieve *(an immutable reference)* to the *leftwards* point character
    /// for [`this set`](Point).
    #[inline]
    pub const fn left_ref(&self) -> &char {
        let &Self { ref left, .. } = self;

        left
    }

    /// Retrieve *(a mutable reference)* to the *leftwards* point character for
    /// [`this set`](Point).
    #[inline]
    pub const fn left_mut(&mut self) -> &mut char {
        let &mut Self { ref mut left, .. } = self;

        left
    }

    /// Retrieve the *rightwards* point character for [`this set`](Point).
    #[inline]
    pub const fn right(&self) -> char {
        let &Self { right, .. } = self;

        right
    }

    /// Retrieve *(an immutable reference)* to the *rightwards* point character
    /// for [`this set`](Point).
    #[inline]
    pub const fn right_ref(&self) -> &char {
        let &Self { ref right, .. } = self;

        right
    }

    /// Retrieve *(a mutable reference)* to the *rightwards* point character for
    /// [`this set`](Point).
    #[inline]
    pub const fn right_mut(&mut self) -> &mut char {
        let &mut Self { ref mut right, .. } = self;

        right
    }
}

impl Default for Point {
    #[inline]
    fn default() -> Self {
        Self::standard()
    }
}

/// A set of arrow characters.
///
/// This is used to convey **directional** meaning in the context of the error
/// report.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Arrow {
    /// Upward-pointing arrow glyph.
    up: char,

    /// Downward-pointing arrow glyph.
    down: char,

    /// Leftward-pointing arrow glyph.
    left: char,

    /// Rightward-pointing arrow glyph.
    right: char,
}

impl Default for Arrow {
    #[inline]
    fn default() -> Self {
        Self::standard()
    }
}

impl Arrow {
    /// A set of *ASCII* arrow characters.
    pub const ASCII: Self = Self::tuple(('^', 'v', '<', '>'));

    /// A set of *unicode* arrow characters.
    pub const UNICODE: Self = Self::tuple(('↑', '↓', '←', '→'));

    /// The standard arrow character set.
    ///
    /// Currently, this is set to [`ASCII`](Arrow::ASCII).
    #[inline]
    pub const fn standard() -> Self {
        Self::ASCII
    }
}

impl Arrow {
    /// Create a new [`Arrow`] from its raw parts: each arrow kind (direction).
    #[inline]
    pub const fn from_raw_parts(up: char, down: char, left: char, right: char) -> Self {
        Self { up, down, left, right }
    }

    /// Create a new [`Arrow`] from a 4-element tuple of each arrow kind
    /// (direction).
    #[inline]
    pub const fn tuple((up, down, left, right): (char, char, char, char)) -> Self {
        Self { up, down, left, right }
    }

    /// Create a new [`Arrow`] from a 4-element array of each arrow kind
    /// (direction).
    #[inline]
    pub const fn array([up, down, left, right]: [char; 4]) -> Self {
        Self { up, down, left, right }
    }

    /// Retrieve the *upwards* arrow character for [`this set`](Arrow).
    #[inline]
    pub const fn up(&self) -> char {
        let &Self { up, .. } = self;

        up
    }

    /// Retrieve *(an immutable reference)* to the *upwards* arrow character for
    /// [`this set`](Arrow).
    #[inline]
    pub const fn up_ref(&self) -> &char {
        let &Self { ref up, .. } = self;

        up
    }

    /// Retrieve *(a mutable reference)* to the *upwards* arrow character for
    /// [`this set`](Arrow).
    #[inline]
    pub const fn up_mut(&mut self) -> &mut char {
        let &mut Self { ref mut up, .. } = self;

        up
    }

    /// Retrieve the *downwards* arrow character for [`this set`](Arrow).
    #[inline]
    pub const fn down(&self) -> char {
        let &Self { down, .. } = self;

        down
    }

    /// Retrieve *(an immutable reference)* to the *downwards* arrow character
    /// for [`this set`](Arrow).
    #[inline]
    pub const fn down_ref(&self) -> &char {
        let &Self { ref down, .. } = self;

        down
    }

    /// Retrieve *(a mutable reference)* to the *downwards* arrow character for
    /// [`this set`](Arrow).
    #[inline]
    pub const fn down_mut(&mut self) -> &mut char {
        let &mut Self { ref mut down, .. } = self;

        down
    }

    /// Retrieve the *leftwards* arrow character for [`this set`](Arrow).
    #[inline]
    pub const fn left(&self) -> char {
        let &Self { left, .. } = self;

        left
    }

    /// Retrieve *(an immutable reference)* to the *leftwards* arrow character
    /// for [`this set`](Arrow).
    #[inline]
    pub const fn left_ref(&self) -> &char {
        let &Self { ref left, .. } = self;

        left
    }

    /// Retrieve *(a mutable reference)* to the *leftwards* arrow character for
    /// [`this set`](Arrow).
    #[inline]
    pub const fn left_mut(&mut self) -> &mut char {
        let &mut Self { ref mut left, .. } = self;

        left
    }

    /// Retrieve the *rightwards* arrow character for [`this set`](Arrow).
    #[inline]
    pub const fn right(&self) -> char {
        let &Self { right, .. } = self;

        right
    }

    /// Retrieve *(an immutable reference)* to the *rightwards* arrow character
    /// for [`this set`](Arrow).
    #[inline]
    pub const fn right_ref(&self) -> &char {
        let &Self { ref right, .. } = self;

        right
    }

    /// Retrieve *(a mutable reference)* to the *rightwards* arrow character for
    /// [`this set`](Arrow).
    #[inline]
    pub const fn right_mut(&mut self) -> &mut char {
        let &mut Self { ref mut right, .. } = self;

        right
    }
}

/// A cycling iterator over a set of distinctive markers.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Marker<'a> {
    /// The base characters to be used as distinctive markers.
    base: &'a [char],

    /// The default character to be used when the marker set is empty.
    default: char,

    /// The current index of the marker for the current generation.
    index: usize,
}

impl<'a> Marker<'a> {
    /// A constant [`Marker`] that uses an exclusively *ASCII* character set.
    pub const ASCII: Self = Self::set(&['^', '*', '#', '~', '@', '-', '+', '%', '!', '?'], '^');

    /// A constant [`Marker`] that uses an exclusively *unicode* character set.
    ///
    /// As of now, this is the same as [`ASCII`](Self::ASCII) since that is good
    /// enough.
    pub const UNICODE: Self = Self::ASCII;

    /// Create a new [`Marker`] with a given base character set.
    #[inline]
    pub const fn set(base: &'a [char], default: char) -> Self {
        Self { base, default, index: 0 }
    }

    /// Retrieve the default character for this distinctive marker set.
    ///
    /// This is the character that is to be used when the marker set is
    /// exhausted.
    #[inline]
    pub const fn default(&self) -> char {
        let &Self { default, .. } = self;

        default
    }
}

impl<'a> Iterator for Marker<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let &mut Self {
            base,
            ref mut index,
            default,
            ..
        } = self;

        match base.get(*index) {
            Some(&c) => {
                *index = (*index + 1) % base.len();

                Some(c)
            }
            None => Some(default),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Marker;

    #[test]
    fn empty_marker_set_uses_default() {
        let mut marker = Marker::set(&[], '?');

        assert_eq!(marker.next(), Some('?'));
        assert_eq!(marker.next(), Some('?'));
    }

    #[test]
    fn marker_set_cycles() {
        let mut marker = Marker::set(&['a', 'b'], '?');

        assert_eq!(marker.next(), Some('a'));
        assert_eq!(marker.next(), Some('b'));
        assert_eq!(marker.next(), Some('a'));
    }
}
