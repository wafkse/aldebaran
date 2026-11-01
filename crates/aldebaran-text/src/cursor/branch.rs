//! Speculative traversal with explicit commitment of cursor progress.
//!
//! See [`Branch`] for further information.

use core::ops::{Deref, DerefMut};

use aldebaran_source::prelude::{SourceDissect, SourceIter};

use super::Cursor;

/// A speculative cursor derived from one parent [`Cursor`].
///
/// The branch dereferences to its independent [`Cursor`]. Consuming the
/// branch has no effect on the parent unless [`Self::backport`] is called, which
/// commits only the final source boundary.
///
/// Nested branches commit only to their immediate parent. Dropping an outer branch also discards any progress committed by its inner branches.
#[derive(Debug)]
// NOTE(invariant): None.
pub struct Branch<'source, 'borrow, S>
where
    'source: 'borrow,
    S: SourceDissect<'source> + SourceIter<'source> + ?Sized,
{
    /// Parent offset updated by [`Self::backport`].
    source_index: &'borrow mut usize,

    /// The currently active branch.
    branch_text: Cursor<'source, S>,
}

impl<'source, 'borrow, S> From<&'borrow mut Cursor<'source, S>> for Branch<'source, 'borrow, S>
where
    'source: 'borrow,
    S: SourceDissect<'source> + SourceIter<'source> + ?Sized,
{
    #[inline]
    fn from(source_text: &'borrow mut Cursor<'source, S>) -> Self {
        let branch_text = *source_text;

        let &mut Cursor { ref mut source_index, .. } = source_text;

        Self { source_index, branch_text }
    }
}

impl<'source, 'borrow, S> Deref for Branch<'source, 'borrow, S>
where
    'source: 'borrow,
    S: SourceDissect<'source> + SourceIter<'source> + ?Sized,
{
    type Target = Cursor<'source, S>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        let &Self { ref branch_text, .. } = self;

        branch_text
    }
}

impl<'source, 'borrow, S> DerefMut for Branch<'source, 'borrow, S>
where
    'source: 'borrow,
    S: SourceDissect<'source> + SourceIter<'source> + ?Sized,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        let &mut Self { ref mut branch_text, .. } = self;

        branch_text
    }
}

impl<'source, 'borrow, S> Branch<'source, 'borrow, S>
where
    'source: 'borrow,
    S: SourceDissect<'source> + SourceIter<'source> + ?Sized,
{
    /// Backport any changes done in this [`Branch`] to the [`Cursor`] we originally diverged from.
    ///
    /// The parent may itself be a speculative branch.
    #[inline]
    pub const fn backport(self) {
        let Self {
            source_index,
            branch_text: Cursor {
                source_index: branch_index,
                ..
            },
        } = self;

        *source_index = branch_index;
    }
}

#[cfg(test)]
mod tests {
    //! Tests module for the [`branch`] module.
    //!
    //! [`branch`]: super

    use crate::cursor::Cursor;

    #[test]
    fn inner_commit_does_not_commit_the_outer_branch() {
        let mut text = Cursor::create("αβ;");

        {
            let mut outer = text.branch();

            assert_eq!(outer.any(), Some('α'));

            let mut inner = outer.branch();

            assert_eq!(inner.any(), Some('β'));
            inner.backport();

            assert_eq!(outer.index(), 4);
            assert_eq!(outer.peek(), Some(';'));
        }

        assert_eq!(text.index(), 0);
        assert_eq!(text.peek(), Some('α'));
    }

    #[test]
    fn outer_commit_discards_uncommitted_inner_progress() {
        let mut text = Cursor::create("αβ;");
        let mut outer = text.branch();

        assert_eq!(outer.any(), Some('α'));

        {
            let mut inner = outer.branch();

            assert_eq!(inner.any(), Some('β'));
            assert_eq!(inner.any(), Some(';'));
        }

        assert_eq!(outer.index(), 2);
        outer.backport();

        assert_eq!(text.index(), 2);
        assert_eq!(text.peek(), Some('β'));
    }
}
