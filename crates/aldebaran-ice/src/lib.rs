#![no_std]
#![doc = include_str!("../../../docs/part/COMPONENT.md")]
#![doc = include_str!("../README.md")]
#![doc = include_str!("../../../docs/part/FACADE.md")]
#![doc = include_str!("../../../docs/part/LICENSE.md")]

use core::{
    fmt,
    hint::{black_box, cold_path},
    marker::PhantomData,
};

/// A type to be used as the central type for raising `ICE`s.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// NOTE(invariant): None.
pub struct Ice<T>(PhantomData<fn() -> T>);

impl<T> Ice<T> {
    /// Raise a message-less `ICE` (internal compiler error).
    #[cold]
    #[inline(never)]
    #[track_caller]
    #[expect(clippy::panic, reason = "this is intended for internal compiler errors")]
    pub const fn raise() -> ! {
        panic!("ICE: internal compiler error");
    }

    /// Raise a messaged `ICE` (internal compiler error) for a specific [`Error`]-like `E`.
    ///
    /// [`Error`]: core::error::Error
    #[cold]
    #[inline(never)]
    #[track_caller]
    #[expect(clippy::unreachable, reason = "this is intended for internal compiler errors")]
    pub fn known<E>(target_error: E) -> !
    where
        E: fmt::Debug,
    {
        black_box(Err::<(), E>(target_error)).expect("ICE: internal compiler error");

        unreachable!()
    }
}

impl<T> Ice<Option<T>> {
    /// Assume that an [`Option`] is always a [`Some`] variant.
    ///
    /// Raises an `ICE` (internal compiler error) if the [`Option`] is a
    /// [`None`] variant.
    #[inline]
    #[track_caller]
    pub fn some(target_value: Option<T>) -> T {
        if let Some(target_value) = target_value {
            target_value
        } else {
            cold_path();

            Self::raise()
        }
    }

    /// Assume that an [`Option`] is always a [`Some`] variant.
    ///
    /// Raises an `ICE` (internal compiler error) if the [`Option`] is a
    /// [`None`] variant, using a message that identifies the state as impossible.
    #[inline]
    #[track_caller]
    pub fn impossible(target_value: Option<T>) -> T {
        if let Some(target_value) = target_value {
            target_value
        } else {
            cold_path();

            unreachable!("ICE: internal compiler error; reached state deemed impossible")
        }
    }
}

impl<T, E> Ice<Result<T, E>> {
    /// Assume that a [`Result`] is always an [`Ok`] variant.
    ///
    /// Raises an `ICE` (internal compiler error) if the [`Result`] is an
    /// [`Err`] variant.
    ///
    /// This is similar to [`Result::expect`], but is used to indicate that
    /// the expectation is internal to the compiler and should never be reached.
    #[inline]
    #[track_caller]
    pub fn ok(target_value: Result<T, E>) -> T
    where
        E: fmt::Debug,
    {
        match target_value {
            Ok(target_value) => target_value,
            Err(target_error) => {
                cold_path();

                Self::known(target_error)
            }
        }
    }
}
