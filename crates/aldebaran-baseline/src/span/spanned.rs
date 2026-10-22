//! Required source extents for values exchanged between processing stages.
//!
//! [`Spanned`] always supplies a nonempty [`Span`]. Values that may have no
//! extent use [`MaybeSpanned`](super::MaybeSpanned) instead, while
//! both contracts leave the coordinate unit and source identity to their users.

use super::Span;

/// A value with a definite [`Span`].
///
/// Implementations and their consumers must agree on the coordinate unit and origin.
pub trait Spanned {
    /// Return this value's span in its associated coordinate system.
    fn span(&self) -> Span;
}

/// A span's associated span is itself.
impl Spanned for Span {
    #[inline]
    fn span(&self) -> Span {
        *self
    }
}
