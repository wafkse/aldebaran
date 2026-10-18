//! Association of report values with semantic metadata subjects.
//!
//! [`Connotate`] lets a diagnostic kind expose structured metadata describing
//! severity and importance. Renderers can therefore style and order reports from
//! structured semantics instead of matching display strings.

/// Maps a value to the semantic subject that consumers use to interpret it.
pub trait Connotate {
    /// Semantic value returned by [`connotates`](Self::connotates).
    type Subject;

    /// Return the semantic value associated with this instance.
    fn connotates(&self) -> Self::Subject;
}
