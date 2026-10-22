//! Content-based source interning with stable, storage-local identities.
//!
//! [`Storage`] permits source borrows, while [`OwnedStorage`] retains values
//! independently of its inputs. Both assign one-based IDs and reuse an ID for
//! values with equal hashes that compare equal under [`SourceDiff::same_as`].
//! For consistent deduplication, source equality and hashing must agree and
//! remain stable while values are stored.

use core::{
    borrow::Borrow,
    convert::TryFrom,
    hash::{BuildHasher, BuildHasherDefault},
};

use alloc::{borrow::Cow, vec::Vec};

use hashbrown::{HashMap, hash_map::RawEntryMut};

use rustc_hash::FxHasher;

use aldebaran_id::{
    ident::TaggedId,
    prelude::{Id, NonZeroed},
};

use crate::{borrow::BorrowInterner, owned::OwnedInterner};

use aldebaran_source::prelude::{SourceDiff, SourceDissect, SourceHash, SourceOwned};

Id!(
    /// Storage-local identity whose default width is 32 bits.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    become StorageId as "storage-id{{{}}}"
);

StorageId!(8);
StorageId!(16);
StorageId!(32 become);
StorageId!(64);

/// Retain source values as borrows lasting for `'a` or as owned representations.
/// Assigned IDs belong to this storage and remain valid across insertions.
#[derive(Debug, Clone)]
// NOTE(invariant): ForAll(Id In IndexedIds): 1 <= Id <= StorageList.Length
// And IndexedHash(Id) = SourceHash(StorageList[Id - 1])
pub struct Storage<'a, S, O, I = StorageId, H = BuildHasherDefault<FxHasher>>
where
    S: SourceDissect<'a> + SourceHash<'a> + SourceDiff<'a> + SourceOwned<'a, Owned = O> + ?Sized,
    H: BuildHasher,
    I: Id,
    I::BackingPrimitive: TryFrom<usize>,
    usize: TryFrom<I::BackingPrimitive>,
{
    /// Source-content hash policy used to identify reusable values.
    build_hasher: H,

    /// IDs grouped by source hash for content-based identity reuse.
    id_existence: HashMap<I, (), ()>,

    /// Retained source representations in ID assignment order.
    storage_list: Vec<Cow<'a, S>>,
}

impl<'a, S, O, I, H> BorrowInterner<'a, S, I> for Storage<'a, S, O, I, H>
where
    S: SourceDissect<'a> + SourceHash<'a> + SourceDiff<'a> + SourceOwned<'a, Owned = O> + ?Sized,
    H: BuildHasher,
    I: Id,
    I::BackingPrimitive: TryFrom<usize>,
    usize: TryFrom<I::BackingPrimitive>,
{
    #[inline]
    fn try_store(&mut self, target_value: Cow<'a, S>) -> Option<I> {
        Self::try_store(self, target_value)
    }

    #[inline]
    fn try_resolve(&self, target_id: I) -> Option<&S> {
        Self::try_resolve(self, target_id)
    }
}

impl<'a, S, O, I, H> Storage<'a, S, O, I, H>
where
    S: SourceDissect<'a> + SourceHash<'a> + SourceDiff<'a> + SourceOwned<'a, Owned = O> + ?Sized,
    H: BuildHasher,
    I: Id,
    I::BackingPrimitive: TryFrom<usize>,
    usize: TryFrom<I::BackingPrimitive>,
{
    /// Create empty storage with the default hasher.
    #[inline]
    pub fn empty() -> Self
    where
        H: Default,
    {
        let build_hasher = H::default();

        let id_existence = HashMap::with_hasher(());

        let storage_list = Vec::new();

        Self {
            build_hasher,
            id_existence,
            storage_list,
        }
    }

    /// Create empty storage using `build_hasher` for source-content hashing.
    #[inline]
    pub const fn with_hasher(build_hasher: H) -> Self {
        let id_existence = HashMap::with_hasher(());

        let storage_list = Vec::new();

        Self {
            build_hasher,
            id_existence,
            storage_list,
        }
    }
}

impl<'a, S, O, I, H> Storage<'a, S, O, I, H>
where
    S: SourceDissect<'a> + SourceHash<'a> + SourceDiff<'a> + SourceOwned<'a, Owned = O> + ?Sized,
    H: BuildHasher,
    I: Id,
    I::BackingPrimitive: TryFrom<usize>,
    usize: TryFrom<I::BackingPrimitive>,
{
    /// Intern a source value, retaining the first representation of equal content.
    /// A distinct value yields `None` without insertion if its new ID cannot be
    /// represented, but already interned values still return their IDs at capacity.
    #[inline]
    pub fn try_store<V>(&mut self, target_value: V) -> Option<I>
    where
        V: Into<Cow<'a, S>>,
    {
        let &mut Self {
            ref build_hasher,
            ref mut id_existence,
            ref mut storage_list,
        } = self;

        let target_value = target_value.into();

        let value_hash = build_hasher.hash_one(target_value.as_ref());

        let raw_entry = id_existence.raw_entry_mut().from_hash(value_hash, |&key| {
            match Self::index(key).and_then(|index| storage_list.get(index)) {
                Some(target_exist) => target_exist.as_ref().same_as(target_value.as_ref()),
                None => false,
            }
        });

        match raw_entry {
            RawEntryMut::Occupied(entry) => {
                let &key = entry.key();

                Some(key)
            }

            RawEntryMut::Vacant(entry) => {
                let key_id: NonZeroed<I::BackingPrimitive> = storage_list
                    .len()
                    .checked_add(1)
                    .and_then(|index| I::BackingPrimitive::try_from(index).ok())
                    .and_then(NonZeroed::new)?;

                let key_id = I::raw(key_id);

                storage_list.push(target_value);

                entry.insert_with_hasher(value_hash, key_id, (), |&key| {
                    match Self::index(key).and_then(|index| storage_list.get(index)) {
                        Some(target_exist) => build_hasher.hash_one(target_exist.as_ref()),
                        None => value_hash,
                    }
                });

                Some(key_id)
            }
        }
    }

    /// Access a retained source by this storage's identity, yielding `None` for
    /// an unassigned or unrepresentable position.
    /// IDs from another storage may resolve to unrelated values here.
    #[inline]
    pub fn try_resolve(&self, target_id: I) -> Option<&S> {
        let &Self { ref storage_list, .. } = self;

        Self::index(target_id).and_then(|index| storage_list.get(index)).map(Cow::as_ref)
    }

    /// Storage position corresponding to a one-based identity, if representable
    /// on this platform, without establishing that a value occupies the position.
    #[inline]
    fn index(target_id: I) -> Option<usize>
    where
        usize: TryFrom<I::BackingPrimitive>,
    {
        usize::try_from(target_id.primitive()).ok()?.checked_sub(1)
    }
}

impl<'a, S, O, I, H> Storage<'a, S, O, I, H>
where
    S: SourceDissect<'a> + SourceHash<'a> + SourceDiff<'a> + SourceOwned<'a, Owned = O> + ?Sized,
    H: BuildHasher,
    I: Id,
    I::BackingPrimitive: TryFrom<usize>,
    usize: TryFrom<I::BackingPrimitive>,
{
    /// Intern under [`Self::try_store`]'s reuse and capacity contract, distinguishing
    /// source types in the returned ID without proving storage membership.
    #[inline]
    pub fn try_store_tagged<V>(&mut self, target_value: V) -> Option<TaggedId<S, I>>
    where
        V: Into<Cow<'a, S>>,
    {
        let target_id = self.try_store(target_value)?;

        Some(TaggedId::new(target_id))
    }

    /// Access a source under [`Self::try_resolve`]'s contract.
    /// The type tag does not establish that this storage issued the ID.
    #[inline]
    pub fn try_resolve_tagged(&self, tagged_id: TaggedId<S, I>) -> Option<&S> {
        self.try_resolve(tagged_id.unwrap())
    }
}

/// Retain source values independently of input borrows under storage-local IDs.
/// Assigned IDs remain valid across insertions, and resolved borrows are tied to this storage.
#[derive(Debug, Clone)]
// NOTE(invariant): ForAll(Id In IndexedIds): 1 <= Id <= StorageList.Length
// And IndexedHash(Id) = SourceHash(StorageList[Id - 1])
pub struct OwnedStorage<S, I = StorageId, H = BuildHasherDefault<FxHasher>>
where
    for<'a> S: SourceDissect<'a> + SourceHash<'a> + SourceDiff<'a> + SourceOwned<'a>,
    H: BuildHasher,
    I: Id,
    I::BackingPrimitive: TryFrom<usize>,
    S: ?Sized,
    usize: TryFrom<I::BackingPrimitive>,
{
    /// Source-content hash policy used to identify reusable values.
    build_hasher: H,

    /// IDs grouped by source hash for content-based identity reuse.
    id_existence: HashMap<I, (), ()>,

    /// Owned source representations retained in ID assignment order.
    storage_list: Vec<S::Owned>,
}

impl<S, I, H> OwnedStorage<S, I, H>
where
    for<'a> S: SourceDissect<'a> + SourceHash<'a> + SourceDiff<'a> + SourceOwned<'a>,
    H: BuildHasher,
    I: Id,
    I::BackingPrimitive: TryFrom<usize>,
    S: ?Sized,
    usize: TryFrom<I::BackingPrimitive>,
{
    /// Create empty storage with the default hasher.
    #[inline]
    pub fn empty() -> Self
    where
        H: Default,
    {
        let build_hasher = H::default();

        let id_existence = HashMap::with_hasher(());

        let storage_list = Vec::new();

        Self {
            build_hasher,
            id_existence,
            storage_list,
        }
    }

    /// Create empty storage using `build_hasher` for source-content hashing.
    #[inline]
    pub const fn with_hasher(build_hasher: H) -> Self {
        let id_existence = HashMap::with_hasher(());

        let storage_list = Vec::new();

        Self {
            build_hasher,
            id_existence,
            storage_list,
        }
    }
}

impl<S, I, H> OwnedInterner<S, I> for OwnedStorage<S, I, H>
where
    for<'a> S: SourceDissect<'a> + SourceHash<'a> + SourceDiff<'a> + SourceOwned<'a>,
    H: BuildHasher,
    I: Id,
    I::BackingPrimitive: TryFrom<usize>,
    S: ?Sized,
    usize: TryFrom<I::BackingPrimitive>,
{
    #[inline]
    fn try_store(&mut self, target_value: &S) -> Option<I> {
        /// An input source borrow that does not require `S` to implement [`AsRef`].
        // NOTE(invariant): None.
        struct Referenced<'a, S>(&'a S)
        where
            S: ?Sized;

        impl<'a, S> AsRef<S> for Referenced<'a, S>
        where
            S: ?Sized,
        {
            #[inline]
            fn as_ref(&self) -> &S {
                let &Self(target_value) = self;

                target_value
            }
        }

        Self::try_store(self, Referenced(target_value))
    }

    #[inline]
    fn try_resolve(&self, target_id: I) -> Option<&S> {
        Self::try_resolve(self, target_id)
    }
}

impl<S, I, H> OwnedStorage<S, I, H>
where
    for<'a> S: SourceDissect<'a> + SourceHash<'a> + SourceDiff<'a> + SourceOwned<'a>,
    H: BuildHasher,
    I: Id,
    I::BackingPrimitive: TryFrom<usize>,
    S: ?Sized,
    usize: TryFrom<I::BackingPrimitive>,
{
    /// Intern an owned source value, retaining the first representation of equal content.
    /// A distinct value yields `None` without insertion if its new ID cannot be
    /// represented, but already interned values still return their IDs at capacity.
    #[inline]
    pub fn try_store<V>(&mut self, target_value: V) -> Option<I>
    where
        V: AsRef<S>,
    {
        let &mut Self {
            ref build_hasher,
            ref mut id_existence,
            ref mut storage_list,
        } = self;

        let target_value = target_value.as_ref();

        let value_hash = build_hasher.hash_one(target_value);

        let raw_entry = id_existence.raw_entry_mut().from_hash(value_hash, |&key| {
            match Self::index(key).and_then(|index| storage_list.get(index)) {
                Some(target_exist) => target_exist.borrow().same_as(target_value),
                None => false,
            }
        });

        match raw_entry {
            RawEntryMut::Occupied(entry) => {
                let &key = entry.key();

                Some(key)
            }

            RawEntryMut::Vacant(entry) => {
                let key_id: NonZeroed<I::BackingPrimitive> = storage_list
                    .len()
                    .checked_add(1)
                    .and_then(|index| I::BackingPrimitive::try_from(index).ok())
                    .and_then(NonZeroed::new)?;

                let key_id = I::raw(key_id);

                storage_list.push(target_value.to_owned());

                entry.insert_with_hasher(value_hash, key_id, (), |&key| {
                    match Self::index(key).and_then(|index| storage_list.get(index)) {
                        Some(target_exist) => build_hasher.hash_one(target_exist.borrow()),
                        None => value_hash,
                    }
                });

                Some(key_id)
            }
        }
    }

    /// Access a retained source by this storage's identity, yielding `None` for
    /// an unassigned or unrepresentable position.
    /// IDs from another storage may resolve to unrelated values here.
    #[inline]
    pub fn try_resolve(&self, target_id: I) -> Option<&S> {
        let &Self { ref storage_list, .. } = self;

        Self::index(target_id).and_then(|index| storage_list.get(index)).map(Borrow::borrow)
    }

    /// Storage position corresponding to a one-based identity, if representable
    /// on this platform, without establishing that a value occupies the position.
    #[inline]
    fn index(target_id: I) -> Option<usize>
    where
        usize: TryFrom<I::BackingPrimitive>,
    {
        usize::try_from(target_id.primitive()).ok()?.checked_sub(1)
    }
}

#[cfg(test)]
mod tests {
    use alloc::{borrow::Cow, format, string::String, vec::Vec};

    use crate::{borrow::BorrowInterner, owned::OwnedInterner};
    use aldebaran_id::{
        Id,
        ident::{Id as IdTrait, TaggedId},
        prelude::NonZeroed,
    };
    use std::collections::hash_map::RandomState;

    use super::{OwnedStorage, Storage, StorageId};

    Id!(
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        become SmallStorageId
    );
    SmallStorageId!(8 become);

    const GROWTH_VALUES: [&str; 12] = [
        "define",
        "radius",
        "area",
        "multiply",
        "conditional",
        "equals",
        "addition",
        "answer",
        "branch",
        "operator",
        "binding",
        "result",
    ];

    #[test]
    fn storage_stores() {
        let mut storage: Storage<'_, str, String, StorageId, RandomState> = Storage::empty();

        let source = "Hello, world!";

        let source_id: StorageId = storage.try_store(source).expect("should never fail");

        assert_eq!(storage.try_resolve(source_id).expect("should never fail"), source);
    }

    #[test]
    fn storage_is_deduplicated() {
        let mut storage: Storage<'_, str, String, StorageId, RandomState> = Storage::empty();

        let source = "Hello, world!";

        let source_id: StorageId = storage.try_store(source).expect("should never fail");
        let source_id2: StorageId = storage.try_store(source).expect("should never fail");

        assert_eq!(storage.try_resolve(source_id).expect("should never fail"), source);

        assert_eq!(source_id, source_id2,);
    }

    #[test]
    fn storage_remains_deduplicated_after_growth() {
        let mut storage: Storage<'_, str, String, StorageId, RandomState> = Storage::empty();
        let mut ids = Vec::new();

        for value in GROWTH_VALUES {
            let id = storage.try_store(value).expect("test storage capacity is sufficient");

            ids.push(id);
        }

        for (value, expected) in GROWTH_VALUES.into_iter().zip(ids) {
            let actual = storage.try_store(value).expect("test storage capacity is sufficient");

            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn owned_storage_remains_deduplicated_after_growth() {
        let mut storage: OwnedStorage<str, StorageId, RandomState> = OwnedStorage::empty();
        let mut ids = Vec::new();

        for value in GROWTH_VALUES {
            let id = storage.try_store(value).expect("test storage capacity is sufficient");

            ids.push(id);
        }

        for (value, expected) in GROWTH_VALUES.into_iter().zip(ids) {
            let actual = storage.try_store(value).expect("test storage capacity is sufficient");

            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn storage_tagged_ids_resolve_the_original_value() {
        let mut storage: Storage<'_, str, String, StorageId, RandomState> = Storage::empty();

        let tagged = storage.try_store_tagged("identifier").expect("test storage capacity is sufficient");

        assert_eq!(storage.try_resolve_tagged(tagged), Some("identifier"));
    }

    #[test]
    fn storage_trait_tagged_ids_resolve_the_original_value() {
        let mut storage: Storage<'_, str, String, StorageId, RandomState> = Storage::empty();
        let input = Cow::Borrowed("identifier");

        let tagged = BorrowInterner::try_store_tagged(&mut storage, input).expect("test storage capacity is sufficient");

        assert_eq!(BorrowInterner::try_resolve_tagged(&storage, tagged), Some("identifier"));
    }

    #[test]
    fn owned_storage_trait_tagged_ids_resolve_the_original_value() {
        let mut storage: OwnedStorage<str, StorageId, RandomState> = OwnedStorage::empty();

        let tagged = OwnedInterner::try_store_tagged(&mut storage, "identifier").expect("test storage capacity is sufficient");

        assert_eq!(OwnedInterner::try_resolve_tagged(&storage, tagged), Some("identifier"));
    }

    #[test]
    fn storage_rejects_ids_outside_its_current_identity_domain() {
        let storage: Storage<'_, str, String, StorageId, RandomState> = Storage::empty();
        let invalid = StorageId::MAX;
        let tagged = TaggedId::<str, StorageId>::new(invalid);

        assert_eq!(storage.try_resolve(invalid), None);
        assert_eq!(storage.try_resolve_tagged(tagged), None);
    }

    #[test]
    fn owned_storage_rejects_ids_outside_its_current_identity_domain() {
        let storage: OwnedStorage<str, StorageId, RandomState> = OwnedStorage::empty();
        let invalid = StorageId::MAX;
        let tagged = TaggedId::<str, StorageId>::new(invalid);

        assert_eq!(storage.try_resolve(invalid), None);
        assert_eq!(OwnedInterner::try_resolve_tagged(&storage, tagged), None);
    }

    #[test]
    fn eight_bit_identity_capacity_is_checked_before_insertion() {
        let mut storage: Storage<'_, str, String, SmallStorageId8, RandomState> = Storage::empty();
        let mut ids = Vec::new();

        for value in 0..u8::MAX {
            let value = format!("value-{value}");

            ids.push(storage.try_store(value).expect("all nonzero u8 IDs fit"));
        }

        assert_eq!(ids.len(), usize::from(u8::MAX));
        assert_eq!(
            storage.try_resolve(*ids.last().expect("at least one value was stored")),
            Some("value-254")
        );
        assert_eq!(storage.try_store("one value too many"), None);
        assert_eq!(
            storage.try_store("value-0"),
            Some(ids[0]),
            "deduplication still succeeds at capacity"
        );
    }

    #[test]
    fn wide_ids_outside_the_platform_index_range_do_not_resolve() {
        let storage: Storage<'_, str, String, super::StorageId64, RandomState> = Storage::empty();
        let invalid = super::StorageId64::raw(NonZeroed::new(u64::MAX).expect("the ID representation is nonzero"));

        assert_eq!(storage.try_resolve(invalid), None);
    }
}
