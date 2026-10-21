//! Annotation markers and optional directional or diagnostic cues.
//!
//! [`Indicator`] groups [`Arrow`], [`Point`], and [`Connote`] with a [`Marker`]
//! sequence. [`Fancy`](super::super::Fancy) currently uses the marker sequence to
//! associate highlighted source spans with their messages. The other cue sets
//! are configurable but are not emitted by that renderer. Glyphs are stored
//! without width checks, so marker alignment requires glyphs that occupy one
//! terminal cell.

/// Annotation marker sequence with separate directional and diagnostic cue sets.
///
/// Fancy copies the stored marker sequence for each excerpt, preserving its
/// starting position across renders. Configuring arrows, points, or diagnostic
/// cues does not change the marker sequence.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
// NOTE(invariant): None.
pub struct Indicator {
    /// Arrows for directional cues, separate from span marker glyphs.
    arrow: Arrow,

    /// Arrowhead-style tips for directional cues without a shaft.
    point: Point,

    /// Symbols identifying warning, error, note, and help meanings.
    connote: Connote,

    /// Repeating symbols that associate highlighted spans with annotation messages.
    marker: Marker<'static>,
}

impl Indicator {
    /// ASCII cues paired with the repeating [`Marker::ASCII`] sequence.
    pub const ASCII: Self = Self::tuple((Arrow::ASCII, Point::ASCII, Connote::ASCII, Marker::ASCII));

    /// Unicode arrows, solid tips, and diagnostic cues with ASCII span markers.
    pub const UNICODE: Self = Self::tuple((Arrow::UNICODE, Point::UNICODE, Connote::UNICODE, Marker::UNICODE));

    /// ASCII cues and markers used by [`Default`].
    #[inline]
    pub const fn standard() -> Self {
        Self::ASCII
    }
}

impl Indicator {
    /// Configure cue families independently of the span marker sequence.
    ///
    /// The marker sequence must borrow static storage because the indicator set
    /// can be retained independently of the source being rendered.
    #[inline]
    pub const fn from_raw_parts(arrow: Arrow, point: Point, connote: Connote, marker: Marker<'static>) -> Self {
        Self {
            arrow,
            point,
            connote,
            marker,
        }
    }

    /// Interpret the entries as arrows, points, diagnostic cues, and span markers.
    ///
    /// Cue families can mix ASCII, Unicode, and custom glyphs.
    #[inline]
    pub const fn tuple((arrow, point, connote, marker): (Arrow, Point, Connote, Marker<'static>)) -> Self {
        Self {
            arrow,
            point,
            connote,
            marker,
        }
    }

    /// Directional cues distinct from the repeating symbols used for spans.
    #[inline]
    pub const fn arrow(&self) -> Arrow {
        let &Self { arrow, .. } = self;

        arrow
    }

    /// Arrow configuration whose Unicode preset uses shafted direction symbols.
    #[inline]
    pub const fn arrow_ref(&self) -> &Arrow {
        let &Self { ref arrow, .. } = self;

        arrow
    }

    /// Customize directional arrows independently of points and span markers.
    #[inline]
    pub const fn arrow_mut(&mut self) -> &mut Arrow {
        let &mut Self { ref mut arrow, .. } = self;

        arrow
    }

    /// Directional tips whose Unicode preset uses solid triangles.
    #[inline]
    pub const fn point(&self) -> Point {
        let &Self { point, .. } = self;

        point
    }

    /// Tip configuration separate from the shafted arrows in [`Arrow`].
    #[inline]
    pub const fn point_ref(&self) -> &Point {
        let &Self { ref point, .. } = self;

        point
    }

    /// Customize directional tips without changing the arrow configuration.
    #[inline]
    pub const fn point_mut(&mut self) -> &mut Point {
        let &mut Self { ref mut point, .. } = self;

        point
    }

    /// Diagnostic meanings represented by warning, error, note, and help symbols.
    #[inline]
    pub const fn connote(&self) -> Connote {
        let &Self { connote, .. } = self;

        connote
    }

    /// Diagnostic cue configuration, including whether note and help symbols are present.
    #[inline]
    pub const fn connote_ref(&self) -> &Connote {
        let &Self { ref connote, .. } = self;

        connote
    }

    /// Replace diagnostic cues independently of directional and span markers.
    #[inline]
    pub const fn connote_mut(&mut self) -> &mut Connote {
        let &mut Self { ref mut connote, .. } = self;

        connote
    }

    /// Span marker sequence copied at its current position.
    ///
    /// Advancing the copy leaves the stored sequence unchanged.
    #[inline]
    pub const fn marker(&self) -> Marker<'static> {
        let &Self { marker, .. } = self;

        marker
    }

    /// Stored sequence position from which each excerpt obtains its marker copy.
    #[inline]
    pub const fn marker_ref(&self) -> &Marker<'static> {
        let &Self { ref marker, .. } = self;

        marker
    }

    /// Change the marker sequence or advance the starting position used by future excerpts.
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

/// Symbols that distinguish diagnostic meanings independently of their text.
///
/// Warning and error symbols are always present. Note and help symbols can be
/// omitted with [`Self::from_raw_parts`], while the tuple and array constructors
/// include all four cues.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
// NOTE(invariant): None.
pub struct Connote {
    /// Symbol for caution rather than a failing operation.
    warning: char,

    /// Symbol identifying an error rather than a warning.
    error: char,

    /// Cue for additional information, absent when no note symbol is configured.
    note: Option<char>,

    /// Cue for corrective advice, absent when no help symbol is configured.
    help: Option<char>,
}

impl Connote {
    /// Warning, error, note, and help symbols spelled `!`, `x`, `i`, and `v`.
    pub const ASCII: Self = Self::tuple(('!', 'x', 'i', 'v'));

    /// Distinct Unicode symbols for warning, error, note, and help meanings.
    pub const UNICODE: Self = Self::tuple(('◈', '❱', '⟐', '⟢'));

    /// ASCII warning and error symbols with note and help cues enabled, as in [`Default`].
    #[inline]
    pub const fn standard() -> Self {
        Self::ASCII
    }
}

impl Connote {
    /// Configure warning and error symbols with independently optional note and help cues.
    #[inline]
    pub const fn from_raw_parts(warning: char, error: char, note: Option<char>, help: Option<char>) -> Self {
        Self {
            warning,
            error,
            note,
            help,
        }
    }

    /// Use warning, error, note, and help symbols in that order, with both optional cues present.
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

    /// Use warning, error, note, and help symbols in that order, including all four meanings.
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

    /// Caution symbol, spelled `!` in the ASCII preset.
    #[inline]
    pub const fn warning(&self) -> char {
        let &Self { warning, .. } = self;

        warning
    }

    /// Configured symbol for a warning rather than an error.
    #[inline]
    pub const fn warning_ref(&self) -> &char {
        let &Self { ref warning, .. } = self;

        warning
    }

    /// Customize the warning symbol independently of the error symbol.
    #[inline]
    pub const fn warning_mut(&mut self) -> &mut char {
        let &mut Self { ref mut warning, .. } = self;

        warning
    }

    /// Error symbol, spelled `x` in the ASCII preset.
    #[inline]
    pub const fn error(&self) -> char {
        let &Self { error, .. } = self;

        error
    }

    /// Configured symbol for a failure rather than a warning.
    #[inline]
    pub const fn error_ref(&self) -> &char {
        let &Self { ref error, .. } = self;

        error
    }

    /// Customize the failure cue while retaining the warning cue.
    #[inline]
    pub const fn error_mut(&mut self) -> &mut char {
        let &mut Self { ref mut error, .. } = self;

        error
    }

    /// Symbol for additional information, with `None` denoting an omitted cue.
    #[inline]
    pub const fn note(&self) -> Option<char> {
        let &Self { note, .. } = self;

        note
    }

    /// Configured note cue, absent when note symbolism is disabled.
    #[inline]
    pub const fn note_ref(&self) -> Option<&char> {
        let &Self { ref note, .. } = self;

        note.as_ref()
    }

    /// Customize an existing note cue.
    ///
    /// An absent cue remains absent because this view does not expose its option.
    #[inline]
    pub const fn note_mut(&mut self) -> Option<&mut char> {
        let &mut Self { ref mut note, .. } = self;

        note.as_mut()
    }

    /// Symbol for corrective advice, with `None` denoting an omitted cue.
    #[inline]
    pub const fn help(&self) -> Option<char> {
        let &Self { help, .. } = self;

        help
    }

    /// Configured help cue, absent when help symbolism is disabled.
    #[inline]
    pub const fn help_ref(&self) -> Option<&char> {
        let &Self { ref help, .. } = self;

        help.as_ref()
    }

    /// Customize an existing help cue without enabling an absent cue.
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

/// Arrowhead-style cues that point toward a position in the output.
///
/// The Unicode preset uses solid triangles, whereas [`Arrow`] uses shafted
/// arrows. Directions are relative to displayed rows and columns.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
// NOTE(invariant): None.
pub struct Point {
    /// Tip directed toward an earlier output row.
    up: char,

    /// Tip directed toward a later output row.
    down: char,

    /// Tip directed toward an earlier column in the same row.
    left: char,

    /// Tip directed toward a later column in the same row.
    right: char,
}

impl Point {
    /// Tips spelled `^`, `v`, `<`, and `>` for up, down, left, and right.
    pub const ASCII: Self = Self::tuple(('^', 'v', '<', '>'));

    /// Solid triangle tips for up, down, left, and right.
    pub const UNICODE: Self = Self::tuple(('▲', '▼', '◀', '▶'));

    /// ASCII directional tips used by [`Default`].
    #[inline]
    pub const fn standard() -> Self {
        Self::ASCII
    }
}

impl Point {
    /// Assign tips for up, down, left, and right independently of the arrow set.
    #[inline]
    pub const fn from_raw_parts(up: char, down: char, left: char, right: char) -> Self {
        Self { up, down, left, right }
    }

    /// Order the tips as up, down, left, and right, relative to output rows and columns.
    #[inline]
    pub const fn tuple((up, down, left, right): (char, char, char, char)) -> Self {
        Self { up, down, left, right }
    }

    /// Order tips as up, down, left, and right, matching [`Self::tuple`].
    #[inline]
    pub const fn array([up, down, left, right]: [char; 4]) -> Self {
        Self { up, down, left, right }
    }

    /// Tip toward an earlier row, represented by `▲` in the Unicode preset.
    #[inline]
    pub const fn up(&self) -> char {
        let &Self { up, .. } = self;

        up
    }

    /// Upward tip whose ASCII spelling is `^`.
    #[inline]
    pub const fn up_ref(&self) -> &char {
        let &Self { ref up, .. } = self;

        up
    }

    /// Customize the tip pointing toward earlier rows.
    #[inline]
    pub const fn up_mut(&mut self) -> &mut char {
        let &mut Self { ref mut up, .. } = self;

        up
    }

    /// Tip toward a later row, represented by `▼` in the Unicode preset.
    #[inline]
    pub const fn down(&self) -> char {
        let &Self { down, .. } = self;

        down
    }

    /// Downward tip whose ASCII spelling is `v`.
    #[inline]
    pub const fn down_ref(&self) -> &char {
        let &Self { ref down, .. } = self;

        down
    }

    /// Customize the tip pointing toward later rows.
    #[inline]
    pub const fn down_mut(&mut self) -> &mut char {
        let &mut Self { ref mut down, .. } = self;

        down
    }

    /// Tip toward an earlier column, represented by `◀` in the Unicode preset.
    #[inline]
    pub const fn left(&self) -> char {
        let &Self { left, .. } = self;

        left
    }

    /// Leftward tip whose ASCII spelling is `<`.
    #[inline]
    pub const fn left_ref(&self) -> &char {
        let &Self { ref left, .. } = self;

        left
    }

    /// Customize the tip pointing toward earlier columns.
    #[inline]
    pub const fn left_mut(&mut self) -> &mut char {
        let &mut Self { ref mut left, .. } = self;

        left
    }

    /// Tip toward a later column, represented by `▶` in the Unicode preset.
    #[inline]
    pub const fn right(&self) -> char {
        let &Self { right, .. } = self;

        right
    }

    /// Rightward tip whose ASCII spelling is `>`.
    #[inline]
    pub const fn right_ref(&self) -> &char {
        let &Self { ref right, .. } = self;

        right
    }

    /// Customize the tip pointing toward later columns.
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

/// Directional cues whose Unicode forms include an arrow shaft.
///
/// The ASCII forms coincide with [`Point`]'s tips. Keeping these configurations
/// separate allows different symbols for an arrow and its tip.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
// NOTE(invariant): None.
pub struct Arrow {
    /// Arrow directed toward an earlier output row.
    up: char,

    /// Arrow directed toward a later output row.
    down: char,

    /// Arrow directed toward an earlier output column.
    left: char,

    /// Arrow directed toward a later output column.
    right: char,
}

impl Default for Arrow {
    #[inline]
    fn default() -> Self {
        Self::standard()
    }
}

impl Arrow {
    /// Direction cues sharing [`Point::ASCII`]'s `^`, `v`, `<`, and `>` spellings.
    pub const ASCII: Self = Self::tuple(('^', 'v', '<', '>'));

    /// Shafted arrows `↑`, `↓`, `←`, and `→` for the four output directions.
    pub const UNICODE: Self = Self::tuple(('↑', '↓', '←', '→'));

    /// ASCII arrows sharing the default point spellings, as used by [`Default`].
    #[inline]
    pub const fn standard() -> Self {
        Self::ASCII
    }
}

impl Arrow {
    /// Assign arrows for up, down, left, and right independently of their tips.
    #[inline]
    pub const fn from_raw_parts(up: char, down: char, left: char, right: char) -> Self {
        Self { up, down, left, right }
    }

    /// Order direction symbols as up, down, left, and right.
    #[inline]
    pub const fn tuple((up, down, left, right): (char, char, char, char)) -> Self {
        Self { up, down, left, right }
    }

    /// Order direction symbols as up, down, left, and right, matching [`Point::array`].
    #[inline]
    pub const fn array([up, down, left, right]: [char; 4]) -> Self {
        Self { up, down, left, right }
    }

    /// Arrow toward an earlier row, represented by `↑` in the Unicode preset.
    #[inline]
    pub const fn up(&self) -> char {
        let &Self { up, .. } = self;

        up
    }

    /// Upward arrow whose ASCII form coincides with [`Point::up`].
    #[inline]
    pub const fn up_ref(&self) -> &char {
        let &Self { ref up, .. } = self;

        up
    }

    /// Customize the upward arrow without altering the separate point set.
    #[inline]
    pub const fn up_mut(&mut self) -> &mut char {
        let &mut Self { ref mut up, .. } = self;

        up
    }

    /// Arrow toward a later row, represented by `↓` in the Unicode preset.
    #[inline]
    pub const fn down(&self) -> char {
        let &Self { down, .. } = self;

        down
    }

    /// Downward arrow whose ASCII form coincides with [`Point::down`].
    #[inline]
    pub const fn down_ref(&self) -> &char {
        let &Self { ref down, .. } = self;

        down
    }

    /// Customize the downward arrow independently of its tip.
    #[inline]
    pub const fn down_mut(&mut self) -> &mut char {
        let &mut Self { ref mut down, .. } = self;

        down
    }

    /// Arrow toward an earlier column, represented by `←` in the Unicode preset.
    #[inline]
    pub const fn left(&self) -> char {
        let &Self { left, .. } = self;

        left
    }

    /// Leftward arrow whose ASCII form coincides with [`Point::left`].
    #[inline]
    pub const fn left_ref(&self) -> &char {
        let &Self { ref left, .. } = self;

        left
    }

    /// Customize the leftward arrow while retaining the separate point set.
    #[inline]
    pub const fn left_mut(&mut self) -> &mut char {
        let &mut Self { ref mut left, .. } = self;

        left
    }

    /// Arrow toward a later column, represented by `→` in the Unicode preset.
    #[inline]
    pub const fn right(&self) -> char {
        let &Self { right, .. } = self;

        right
    }

    /// Rightward arrow whose ASCII form coincides with [`Point::right`].
    #[inline]
    pub const fn right_ref(&self) -> &char {
        let &Self { ref right, .. } = self;

        right
    }

    /// Customize the rightward arrow independently of its tip.
    #[inline]
    pub const fn right_mut(&mut self) -> &mut char {
        let &mut Self { ref mut right, .. } = self;

        right
    }
}

/// An endless sequence of symbols for distinguishing annotation spans.
///
/// Nonempty sequences repeat from their beginning after the last symbol. An
/// empty sequence repeats the fallback forever, so iteration never returns
/// `None`. Copying a marker preserves its current position and allows the copy
/// to advance independently.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
// NOTE(invariant): `index` is zero for an empty `base` and otherwise is strictly less than `base.len()`.
pub struct Marker<'a> {
    /// Borrowed symbols repeated in their supplied order.
    base: &'a [char],

    /// Repeated fallback symbol when `base` is empty.
    default: char,

    /// Position of the next symbol in `base`, or zero when the sequence is empty.
    index: usize,
}

impl<'a> Marker<'a> {
    /// Ten repeating ASCII symbols, starting with `^` and using it as the fallback.
    pub const ASCII: Self = Self::set(&['^', '*', '#', '~', '@', '-', '+', '%', '!', '?'], '^');

    /// The same sequence as [`Self::ASCII`], keeping span markers stable across presets.
    pub const UNICODE: Self = Self::ASCII;

    /// Begin a repeating sequence at its first symbol.
    ///
    /// An empty `base` produces `default` indefinitely rather than ending iteration.
    #[inline]
    pub const fn set(base: &'a [char], default: char) -> Self {
        Self { base, default, index: 0 }
    }

    /// Fallback symbol repeated when the configured sequence is empty.
    ///
    /// A nonempty sequence cycles and never reaches this fallback through exhaustion.
    #[inline]
    pub const fn default(&self) -> char {
        let &Self { default, .. } = self;

        default
    }
}

impl<'a> Iterator for Marker<'a> {
    type Item = char;

    #[inline]
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
