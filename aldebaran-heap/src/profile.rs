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
