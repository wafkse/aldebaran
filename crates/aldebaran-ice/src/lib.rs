#![no_std]
#![doc = include_str!("../README.md")]

use core::{error::Error, fmt, hint::black_box, marker::PhantomData};

/// Panic boundary for violated compiler assumptions.
///
/// The type parameter selects helpers for plain failure, optional values, or
/// results without storing a value. Callers must establish that missing or
/// erroneous values indicate an internal defect, since these helpers panic
/// instead of producing source diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// NOTE(invariant): None.
pub struct Ice<T>(PhantomData<fn() -> T>);

impl<T> Ice<T> {
    /// Signal an internal failure without requiring an error value.
    /// The panic uses the fixed internal compiler error message.
    #[cold]
    #[expect(clippy::panic, reason = "this is intended for internal compiler errors")]
    #[inline(never)]
    pub const fn raise() -> ! {
        panic!("ICE: internal compiler error");
    }

    /// Panic with an internal compiler error message that includes `target_error`'s [`fmt::Debug`] representation.
    #[cold]
    #[inline(never)]
    pub fn known<E>(target_error: E) -> !
    where
        E: Error,
    {
        black_box(Err::<(), E>(target_error)).expect("ICE: internal compiler error");

        unreachable!()
    }
}

impl<T> Ice<Option<T>> {
    /// Require a value promised by a compiler invariant.
    /// Absence panics with the fixed internal compiler error message.
    #[inline]
    pub fn unwrap(target_value: Option<T>) -> T {
        match target_value {
            Some(value) => value,
            None => Self::raise(),
        }
    }

    /// Require a value when absence represents an impossible compiler state.
    /// The panic identifies that state without including the option's debug representation.
    #[inline]
    pub fn impossible(target_value: Option<T>) -> T {
        match target_value {
            Some(value) => value,
            None => unreachable!("ICE: internal compiler error; reached state deemed impossible"),
        }
    }
}

impl<T, E> Ice<Result<T, E>> {
    /// Require success from an operation assumed infallible by its compiler caller.
    /// Failure panics with an internal expectation message and the error's debug representation.
    #[inline]
    pub fn expect(target_value: Result<T, E>) -> T
    where
        E: fmt::Debug,
    {
        target_value.expect("ICE: internal expectation was not met")
    }
}
