//! Owning interner storage.
//!
//! See [`OwnedInterner`] for more information.

use alloc::borrow::ToOwned;

use aldebaran_id::ident::{Id, TaggedId};

/// A trait for interner storage that owns values independently of the input
/// borrow. This is useful when a value must outlive its source. A successful key
/// resolves through the interner that returned it, while equal inputs need not
/// receive the same key.
pub trait OwnedInterner<B, K>
where
    B: ToOwned + ?Sized,
    K: Id,
{
    /// Attempt to store the target value in the interner.
    ///
    /// If internment succeeds, return the associated key `K`. If no key can be
    /// assigned, this method returns [`None`].
    fn try_store(&mut self, target_value: &B) -> Option<K>;

    /// Try to resolve `target_id` to a reference to a stored value. A key from
    /// another interner may fail to resolve or may identify a different value.
    fn try_resolve(&self, target_id: K) -> Option<&B>;

    /// Attempt to store the target value and return a [`TaggedId`] tagged with
    /// its type. The type tag does not identify the interner that issued the key.
    #[inline]
    fn try_store_tagged(&mut self, target_value: &B) -> Option<TaggedId<B, K>> {
        self.try_store(target_value).map(TaggedId::new)
    }

    /// Try to resolve a [`TaggedId`] to a reference to the stored value. The tag
    /// distinguishes the value type but does not identify the issuing interner.
    #[inline]
    fn try_resolve_tagged(&self, tagged_id: TaggedId<B, K>) -> Option<&B> {
        self.try_resolve(tagged_id.unwrap())
    }
}
