//! Color, style, and deferred painting primitives for ANSI output.
//!
//! See [`color::Color`], [`style::Style`], and [`paint::Painted`] for further information.

pub mod color;
pub mod paint;
pub mod style;

pub use paint::Paintable;
