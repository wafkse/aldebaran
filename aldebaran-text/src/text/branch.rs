//! Speculative branching for [`Text`] cursors.
//!
//! A [`Branch`] copies the active cursor while retaining mutable access to the
//! parent boundary. Work remains isolated until [`Branch::backport`] is called,
//! allowing failed lexical alternatives to be discarded without restoration code.

use core::ops::{Deref, DerefMut};

use aldebaran_source::prelude::{SourceDissect, SourceIter};

use super::Text;

/// A speculative cursor derived from one parent [`Text`].
///
/// The branch dereferences to its independent [`Text`] cursor. Consuming the
/// branch has no effect on the parent unless [`Self::backport`] is called, which
/// commits only the final source boundary.
#[derive(Debug)]
pub struct Branch<'source, 'borrow, S>
where
    'source: 'borrow,
    S: SourceDissect<'source> + SourceIter<'source> + ?Sized,
{
    /// A mutable reference to the [`Text::source_index`] that branch originates from.
    source_index: &'borrow mut usize,

    /// The currently active branch.
    branch_text: Text<'source, S>,
}

impl<'source, 'borrow, S> From<&'borrow mut Text<'source, S>> for Branch<'source, 'borrow, S>
where
    'source: 'borrow,
    S: SourceDissect<'source> + SourceIter<'source> + ?Sized,
{
    #[inline]
    fn from(source_text: &'borrow mut Text<'source, S>) -> Self {
        let branch_text = source_text.clone();

        let &mut Text { ref mut source_index, .. } = source_text;

        Self { source_index, branch_text }
    }
}

impl<'source, 'borrow, S> Deref for Branch<'source, 'borrow, S>
where
    'source: 'borrow,
    S: SourceDissect<'source> + SourceIter<'source> + ?Sized,
{
    type Target = Text<'source, S>;

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
    /// Backport any changes done in this [`Branch`] to the [`Text`] we originally diverged from.
    #[inline]
    pub const fn backport(self) {
        let Self {
            source_index,
            branch_text: Text {
                source_index: branch_index,
                ..
            },
        } = self;

        *source_index = branch_index;
    }
}
