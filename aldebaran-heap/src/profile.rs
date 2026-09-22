//! Allocation profile identities and profiler hooks.
//!
//! Profiles provide compact allocator identities, while [`Profiler`] supplies
//! optional human-readable names consumed by the heap allocator wrapper.

use core::num::NonZero;

/// The underlying storage type for a [`Profile`].
///
/// This is simple a profile ID.
pub type Profiled = usize;

/// A singular allocation profile.
///
/// A profile is simply a non-zero integer representing a profiled identity.
pub enum Profile {
    /// No profile.
    None,

    /// An existing profile.
    Exist(NonZero<Profiled>),
}

impl Profile {
    /// Reinterpret this [`Profile`] as a [`Profiled`].
    #[inline]
    pub const fn be(self) -> Profiled {
        // SAFETY: This simply reinterprets the current Profile as a Profiled
        // type. This is inherently safe because if niche
        // optimizations are not applied, transmutation would result in a
        // compilation error due to mismatching sizes.
        // *NOTE[NOOP]: This should compile down to a no-op.
        unsafe { core::mem::transmute::<Profile, Profiled>(self) }
    }

    /// Reinterpret this [`Profiled`] as a [`Profile`].
    #[inline]
    pub const fn is(value: Profiled) -> Profile {
        // SAFETY: This simply reinterprets the current Profiled as a Profile
        // type. This is inherently safe because if niche
        // optimizations are not applied, transmutation would result in a
        // compilation error due to mismatching sizes.
        // *NOTE[NOOP]: This should compile down to a no-op.
        unsafe { core::mem::transmute::<Profiled, Profile>(value) }
    }

    /// The default, null-pointing [`Profiled`].
    #[inline]
    pub const fn none() -> Profiled {
        Profile::be(Profile::None)
    }
}

/// A trait for profiler drivers.
pub trait Profiler: Send + Sync {
    /// Retrieve the name corresponding to the given [`Profile`].
    fn name(&self, profile: Profile) -> Option<&'static str>;
}

#[cfg(test)]
mod tests {
    use core::num::NonZero;

    use super::{Profile, Profiled};

    #[test]
    fn none_uses_zero_profile_identity() {
        assert_eq!(Profile::none(), 0);

        match Profile::is(0) {
            Profile::None => {}
            Profile::Exist(_) => panic!("zero must decode as the absent profile"),
        }
    }

    #[test]
    fn existing_profiles_round_trip_through_storage_identity() {
        for raw in [1, 2, 17, Profiled::MAX] {
            let nonzero = NonZero::new(raw).expect("test profile identities are nonzero");
            let stored = Profile::be(Profile::Exist(nonzero));

            assert_eq!(stored, raw);

            match Profile::is(stored) {
                Profile::Exist(decoded) => assert_eq!(decoded, nonzero),
                Profile::None => panic!("nonzero profile identity must remain present"),
            }
        }
    }
}
