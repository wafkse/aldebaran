//! Association of report values with semantic metadata subjects.
//!
//! [`Connotate`] lets a diagnostic kind expose structured metadata describing
//! severity and importance. Renderers can therefore style and order reports from
//! structured semantics instead of matching display strings.

/// Maps a value to the semantic subject that consumers use to interpret it.
pub trait Connotate {
    /// The connotative subject.
    ///
    /// This is the meaning that is being connotated to the outsider.
    type Subject;

    /// Determine the [`meaning`](Connotate::Subject) that is being connotated.
    fn connotates(&self) -> Self::Subject;
}
