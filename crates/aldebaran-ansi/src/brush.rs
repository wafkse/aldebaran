//! Deferred terminal presentation built from colors and text attributes.
//!
//! [`color::Color`] selects a palette or RGB representation and [`style::Style`]
//! combines those choices with text attributes. [`Paintable`] attaches that
//! style to printable values for emission when they are rendered.

pub mod color;
pub mod paint;
pub mod style;

pub use paint::Paintable;
