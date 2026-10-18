//! Interning whose stored values outlive their input borrows.
//!
//! [`OwnedInterner`] makes implementations responsible for retaining a value that
//! later lookups can borrow, allowing the caller's input to be released.

use alloc::borrow::ToOwned;

use aldebaran_id::ident::{Id, TaggedId};

/// Retain values independently of input borrows under keys resolvable by the same interner.
/// Successful storage promises lookup of the stored value, while equal inputs
/// need not share a key.
pub trait OwnedInterner<B, K>
where
    B: ToOwned + ?Sized,
    K: Id,
{
    /// Store a value, returning `None` when no key can be assigned.
    fn try_store(&mut self, target_value: &B) -> Option<K>;

    /// Resolve a key in this interner. Keys from another interner may fail or
    /// identify a different value.
    fn try_resolve(&self, target_id: K) -> Option<&B>;

    /// Store under the same success and failure contract as [`Self::try_store`],
    /// adding a value-type distinction that does not identify the issuing interner.
    #[inline]
    fn try_store_tagged(&mut self, target_value: &B) -> Option<TaggedId<B, K>> {
        self.try_store(target_value).map(TaggedId::new)
    }

    /// Resolve under [`Self::try_resolve`]'s contract, including its restriction
    /// to the issuing interner despite the value-type tag.
    #[inline]
    fn try_resolve_tagged(&self, tagged_id: TaggedId<B, K>) -> Option<&B> {
        self.try_resolve(tagged_id.into_inner())
    }
}
