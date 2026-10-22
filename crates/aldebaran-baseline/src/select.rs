//! Selection enums retain which of two alternatives produced a value, including
//! whether both alternatives are present when combining optional results.

mod either;
mod either_or_both;

pub use self::{either::Either, either_or_both::EitherOrBoth};
