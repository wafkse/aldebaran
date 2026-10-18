//! Rendering stages that compose the fancy textual diagnostic backend.
//!
//! [`Header`] writes report identity and location information. [`ByLines`] then
//! projects source extents and annotations line by line. Keeping these stages
//! separate lets layout state remain local to the part of rendering that owns it.

mod header;

mod lines;

pub use self::{header::Header, lines::ByLines};
