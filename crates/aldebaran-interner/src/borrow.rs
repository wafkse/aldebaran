//! Borrowing interner storage.
//!
//! See [`BorrowInterner`] for more information.

use alloc::borrow::{Cow, ToOwned};

use aldebaran_id::ident::{Id, TaggedId};

/// An trait for interner storage that may borrow from an existing source.
///
/// This behavior happens to be useful when something needs to be copied verbatim from a source, but without further duplication.
pub trait BorrowInterner<'a, B, K>
where
    B: ToOwned + ?Sized + 'a,
    K: Id,
{
    /// Attempt to store the value `V` in the interner.
    ///
    /// If internment succeeds, return the associated key `K`.
    fn try_store(&mut self, target_value: Cow<'a, B>) -> Option<K>;

    /// Try to resolve `K` to a reference to `V` inside the interner.
    fn try_resolve(&self, target_id: K) -> Option<&B>;

    /// Attempt to store a the target value `V` in this interner and return an [`Id`] that is tagged with the target type.
    #[inline]
    fn try_store_tagged(&mut self, target_value: Cow<'a, B>) -> Option<TaggedId<B, K>> {
        self.try_store(target_value).map(TaggedId::new)
    }

    /// Try to resolve a [`TaggedId`] to a reference to the stored value.
    #[inline]
    fn try_resolve_tagged(&self, tagged_id: TaggedId<B, K>) -> Option<&B> {
        self.try_resolve(tagged_id.into_inner())
    }
}
