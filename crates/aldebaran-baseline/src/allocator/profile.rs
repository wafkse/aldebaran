//! Profile identities and optional names for allocator debug output.
//!
//! [`Profile`] distinguishes an absent label from a nonzero identity, while
//! [`Profiled`] carries that choice in a [`Heap`](super::heap::Heap) const
//! parameter. [`Profiler`] supplies names without assigning allocation policy.

use core::num::NonZero;

/// Scalar type used by the [`Heap`](super::heap::Heap) profile const parameter.
pub type Profiled = usize;

/// A profile label represented by either the no-profile case or a nonzero ID.
///
/// [`Self::be`] encodes [`Self::None`] as zero and preserves the ID of
/// [`Self::Exist`], while [`Self::is`] recovers the corresponding label. This
/// allows [`Heap`](super::heap::Heap) to select a profile through a scalar const
/// parameter without depending on the enum's memory layout.
///
/// Although these conversions preserve the label, the scalar representation is
/// intended for allocator configuration and must not be used for persistence or
/// interchange.
pub enum Profile {
    /// The no-profile label.
    None,

    /// A nonzero profile identifier.
    Exist(NonZero<Profiled>),
}

impl Profile {
    /// Convert a profile label to the scalar used by [`Heap`](super::heap::Heap).
    #[inline]
    pub const fn be(self) -> Profiled {
        match self {
            Self::None => 0,
            Self::Exist(value) => value.get(),
        }
    }

    /// Interpret a [`Heap`](super::heap::Heap) profile scalar as a profile label.
    #[inline]
    pub const fn is(value: Profiled) -> Profile {
        match NonZero::new(value) {
            Some(value) => Self::Exist(value),
            None => Self::None,
        }
    }

    /// The scalar representation of [`Profile::None`].
    #[inline]
    pub const fn none() -> Profiled {
        Profile::be(Profile::None)
    }
}

/// Supplies static names for profile labels in allocator debug output.
pub trait Profiler: Send + Sync {
    /// Return a process-lifetime name for a known profile, if available.
    fn name(&self, profile: Profile) -> Option<&'static str>;
}

#[cfg(test)]
mod tests {
    use super::{Profile, Profiled};

    #[test]
    fn scalar_conversion_preserves_absence_and_nonzero_ids() {
        const ABSENT: Profiled = Profile::none();
        const MAXIMUM: Profile = Profile::is(Profiled::MAX);

        assert_eq!(ABSENT, 0);
        assert!(matches!(Profile::is(ABSENT), Profile::None));
        assert!(matches!(MAXIMUM, Profile::Exist(value) if value.get() == Profiled::MAX));

        for scalar in [0, 1, 2, Profiled::MAX] {
            assert_eq!(Profile::is(scalar).be(), scalar);
        }
    }
}
