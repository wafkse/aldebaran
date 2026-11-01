//! Borrowing interner storage.
//!
//! See [`BorrowInterner`] for further information.

use alloc::borrow::{Cow, ToOwned};

use aldebaran_id::ident::{Id, TaggedId};

/// A trait for interner storage that may borrow from an existing source.
///
/// This is useful when a value can be copied verbatim from a source without
/// duplicating its contents. A successful key resolves through the interner that
/// returned it, although equal inputs do not have to receive the same key.
pub trait BorrowInterner<'a, B, K>
where
    B: ToOwned + ?Sized + 'a,
    K: Id,
{
    /// Attempt to store the target value in the interner.
    ///
    /// If internment succeeds, return the associated key `K`. If no key can be
    /// assigned, this method returns [`None`].
    fn try_store(&mut self, target_value: Cow<'a, B>) -> Option<K>;

    /// Try to resolve `target_id` to a reference to a stored value. A key from
    /// another interner may fail to resolve or may identify a different value.
    fn try_resolve(&self, target_id: K) -> Option<&B>;

    /// Attempt to store the target value and return a [`TaggedId`] tagged with
    /// its type. The type tag does not identify the interner that issued the key.
    #[inline]
    fn try_store_tagged(&mut self, target_value: Cow<'a, B>) -> Option<TaggedId<B, K>> {
        self.try_store(target_value).map(TaggedId::new)
    }

    /// Try to resolve a [`TaggedId`] to a reference to the stored value. The tag
    /// distinguishes the value type but does not identify the issuing interner.
    #[inline]
    fn try_resolve_tagged(&self, tagged_id: TaggedId<B, K>) -> Option<&B> {
        self.try_resolve(tagged_id.unwrap())
    }
}
