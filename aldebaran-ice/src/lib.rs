#![no_std]
#![doc = include_str!("../README.md")]

use core::{error::Error, fmt, hint::black_box, marker::PhantomData};

/// An uninhabited type to be used as the central type for raising `ICE`s.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ice<T>(PhantomData<fn() -> T>);

impl<T> Ice<T> {
    /// Raise a message-less `ICE` (internal compiler error).
    #[cold]
    #[expect(clippy::panic, reason = "this is intended for internal compiler errors")]
    #[inline(never)]
    pub const fn raise() -> ! {
        panic!("ICE: internal compiler error");
    }

    /// Raise a messaged `ICE` (internal compiler error) for a specific [`Error`] `E`.
    #[cold]
    #[inline(never)]
    pub fn known<E>(target_error: E) -> !
    where
        E: Error,
    {
        // NOTE: Don't want the compiler
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
    pub fn unwrap(target_value: Option<T>) -> T {
        match target_value {
            Some(value) => value,
            None => Self::raise(),
        }
    }

    /// Assume that an [`Option`] is always a [`Some`] variant.
    ///
    /// Raises an `ICE` (internal compiler error) if the [`Option`] is a
    /// [`None`] variant, but does not include the error message.
    #[inline]
    pub fn impossible(target_value: Option<T>) -> T {
        match target_value {
            Some(value) => value,
            None => unreachable!("ICE: internal compiler error; reached state deemed impossible"),
        }
    }
}

impl<T, E> Ice<Result<T, E>> {
    /// Assume that a [`Result`] is always an [`Ok`] variant.
    ///
    /// Raises an `ICE` (internal compiler error) if the [`Result`] is an
    /// [`Err`] variant.
    ///
    /// This is exactly the same as [`Result::expect`], but is used to indicate that
    /// the expectation is internal to the compiler and should never be reached.
    #[inline]
    pub fn expect(target_value: Result<T, E>) -> T
    where
        E: fmt::Debug,
    {
        target_value.expect("ICE: internal expectation was not met")
    }
}
