//! Unified interning storage for source-derived values.
//!
//! [`Storage`] can retain borrowed source slices and owned transformed values in
//! one identity domain. [`OwnedStorage`] provides the corresponding fully owned
//! model when values must outlive any particular source borrow.

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
    /// A general-purpose storage identifier.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    become StorageId as "storage-id{{{}}}"
);

StorageId!(8);
StorageId!(16);
StorageId!(32 become);
StorageId!(64);

/// A storage for source-based data.
///
/// This is the central storage type for interned data that is either:
///
/// - A chunk of the source itself.
///
/// - A value that was partially/totally derived from the source.
///
/// Therefore, to allow for this to be possible, owned values must be able to
/// convert to the same kind of source reference for storage to be able to
/// have a consistent outwards interface.
#[derive(Debug, Clone)]
pub struct Storage<'a, S, O, I = StorageId, H = BuildHasherDefault<FxHasher>>
where
    S: SourceDissect<'a> + SourceHash<'a> + SourceDiff<'a> + SourceOwned<'a, Owned = O> + ?Sized,
    H: BuildHasher,
    I: Id,
    I::BackingPrimitive: TryFrom<usize>,
    usize: TryFrom<I::BackingPrimitive>,
{
    /// The hasher builder for this storage.
    ///
    /// Used for value internment.
    build_hasher: H,

    /// The hash map that stores the interned values.
    ///
    /// This [`HashMap`] has no ability to hash keys by itself, so [`Storage`]
    /// needs to provide a hasher.
    id_existence: HashMap<I, (), ()>,

    /// The storage vector that stores the interned values.
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
    /// A new, completely empty internment storage.
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

    /// A new internment storage with a custom hasher builder.
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
    /// Store either an owned value or a reference to a value in this
    /// [`Storage`].
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

    /// Try to resolve a [`StorageId`] to a reference to the stored value.
    ///
    /// If the target [`trait@Id`] `I` was indeed sourced from this [`Storage`], then
    /// this function will never fail.
    #[inline]
    pub fn try_resolve(&self, target_id: I) -> Option<&S> {
        let &Self { ref storage_list, .. } = self;

        Self::index(target_id).and_then(|index| storage_list.get(index)).map(Cow::as_ref)
    }

    /// Convert a one-based identifier into its zero-based storage position.
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
    /// Store a the target value `V` in this [`Storage`] and return a
    /// [`TaggedId`] that is associated with the target value.
    #[inline]
    pub fn try_store_tagged<V>(&mut self, target_value: V) -> Option<TaggedId<S, I>>
    where
        V: Into<Cow<'a, S>>,
    {
        let target_id = self.try_store(target_value)?;

        Some(TaggedId::new(target_id))
    }

    /// Try to resolve a [`TaggedId`] to a reference to the stored value.
    #[inline]
    pub fn try_resolve_tagged(&self, tagged_id: TaggedId<S, I>) -> Option<&S> {
        self.try_resolve(tagged_id.into_inner())
    }
}

/// A storage for source-based data.
///
/// This is the central storage type for interned data that is either:
///
/// - A chunk of the source itself.
///
/// - A value that was partially/totally derived from the source.
///
/// Therefore, to allow for this to be possible, owned values must be able to
/// convert to the same kind of source reference for storage to be able to
/// have a consistent outwards interface.
#[derive(Debug, Clone)]
pub struct OwnedStorage<S, I = StorageId, H = BuildHasherDefault<FxHasher>>
where
    for<'a> S: SourceDissect<'a> + SourceHash<'a> + SourceDiff<'a> + SourceOwned<'a>,
    H: BuildHasher,
    I: Id,
    I::BackingPrimitive: TryFrom<usize>,
    S: ?Sized,
    usize: TryFrom<I::BackingPrimitive>,
{
    /// The hasher builder for this storage.
    ///
    /// Used for value internment.
    build_hasher: H,

    /// The hash map that stores the interned values.
    ///
    /// This [`HashMap`] has no ability to hash keys by itself, so [`Storage`]
    /// needs to provide a hasher.
    id_existence: HashMap<I, (), ()>,

    /// The storage vector that stores the interned values.
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
    /// A new, completely empty internment storage.
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

    /// A new internment storage with a custom hasher builder.
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
    /// Store the target value `V` in this [`OwnedStorage`].
    ///
    /// Returns the [`StorageId`] that is associated with the target value, if internment did
    /// not fail.
    ///
    /// # Remarks
    ///
    /// Note that failure is very unlikely, although it can realistically happen if the used [`trait@Id`] type is too small to
    /// represent the amount of values that are to be stored in this [`OwnedStorage`].
    ///
    /// If your target [`trait@Id`] type is big enough, then this function will never realistically fail.
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

    /// Try to resolve a [`StorageId`] to a reference to the stored value.
    ///
    /// If the target [`trait@Id`] `I` was indeed sourced from this [`OwnedStorage`], this method will never fail.
    #[inline]
    pub fn try_resolve(&self, target_id: I) -> Option<&S> {
        let &Self { ref storage_list, .. } = self;

        Self::index(target_id).and_then(|index| storage_list.get(index)).map(Borrow::borrow)
    }

    /// Convert a one-based identifier into its zero-based storage position.
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
