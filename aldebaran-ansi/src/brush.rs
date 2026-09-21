//! Color, style, and deferred painting primitives for ANSI output.
//!
//! Color models describe terminal color identity, [`Style`] combines color and
//! text attributes, and [`Painted`] applies that style only while a printable
//! subject is written. The module keeps presentation policy separate from data.

pub mod color;
pub mod paint;
pub mod style;

pub use paint::Paintable;
