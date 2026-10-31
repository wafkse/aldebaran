//! Interning that may retain source borrows alongside owned values.
//!
//! [`BorrowInterner`] accepts either ownership form under the same lookup contract,
//! allowing borrowed inputs to remain tied to their source lifetime.

use alloc::borrow::{Cow, ToOwned};

use aldebaran_id::ident::{Id, TaggedId};

/// An trait for interner storage that may borrow from an existing source.
///
/// Successful storage returns a key that resolves to the stored value in this interner. Equal inputs do not have to receive the same key.
pub trait BorrowInterner<'a, B, K>
where
    B: ToOwned + ?Sized + 'a,
    K: Id,
{
    /// Store a value, returning `None` when no key can be assigned.
    fn try_store(&mut self, target_value: Cow<'a, B>) -> Option<K>;

    /// Resolve a key in this interner. Keys from another interner may fail or
    /// identify a different value.
    fn try_resolve(&self, target_id: K) -> Option<&B>;

    /// Store under the same success and failure contract as [`Self::try_store`],
    /// adding a value-type distinction that does not identify the issuing interner.
    #[inline]
    fn try_store_tagged(&mut self, target_value: Cow<'a, B>) -> Option<TaggedId<B, K>> {
        self.try_store(target_value).map(TaggedId::new)
    }

    /// Resolve under [`Self::try_resolve`]'s contract, including its restriction
    /// to the issuing interner despite the value-type tag.
    #[inline]
    fn try_resolve_tagged(&self, tagged_id: TaggedId<B, K>) -> Option<&B> {
        self.try_resolve(tagged_id.unwrap())
    }
}
