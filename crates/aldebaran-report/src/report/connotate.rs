//! Association of report values with semantic metadata subjects.
//!
//! [`Connotate`] lets a diagnostic kind expose a stable metadata value describing
//! severity and importance. Renderers can therefore style and order reports from
//! structured semantics instead of matching display strings.

/// A trait that connotates a subject with a semantic meaning through a [`connotative subject`].
///
/// [`connotative subject`]: Connotate::Subject
pub trait Connotate {
    /// The connotative subject.
    ///
    /// This is the meaning that is being connotated to the outsider.
    type Subject;

    /// Determine the [`meaning`](Connotate::Subject) that is being connotated.
    fn connotates(&self) -> Self::Subject;
}
