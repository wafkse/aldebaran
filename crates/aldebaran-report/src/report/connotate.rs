//! Metadata associated with diagnostic kinds.
//!
//! See [`Connotate`] for further information.

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
