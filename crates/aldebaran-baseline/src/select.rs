//! The module hosting various "selection" types.
//!
//! See [`Either`] and [`EitherOrBoth`] for further information.

mod either;
mod either_or_both;

pub use self::{either::Either, either_or_both::EitherOrBoth};
