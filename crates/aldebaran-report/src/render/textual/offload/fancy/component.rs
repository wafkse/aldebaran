//! Components that present a diagnostic heading and its source excerpt.
//!
//! The heading establishes the diagnostic context before [`ByLines`] lays out
//! the annotated source.

mod header;

mod lines;

pub use self::{header::Header, lines::ByLines};
