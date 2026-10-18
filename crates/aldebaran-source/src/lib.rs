#![doc = include_str!("../README.md")]
#![no_std]

extern crate alloc;

pub mod component;
pub mod debug;
pub mod diff;
pub mod dissect;
pub mod hash;
pub mod iter;
pub mod line;
pub mod location;
pub mod metadata;
pub mod owned;
pub mod terminate;

pub mod source;
pub mod standard;

mod bytes;
mod str;

pub mod prelude {
    //! The `aldebaran-source` prelude.
    //!
    //! Re-exports the most commonly used items from this crate.

    pub use crate::component::{AsciiByte, AsciiComponent, Component, DigitComponent, TextComponent};

    pub use crate::diff::SourceDiff;

    pub use crate::dissect::SourceDissect;

    pub use crate::hash::SourceHash;

    pub use crate::iter::SourceIter;

    pub use crate::line::{LineBreak, LineContent, LineExtent, LineHandle, LineId, LineSegmented, SourceLines};

    pub use crate::location::Location;

    pub use crate::metadata::SourceMetadata;

    pub use crate::source::Source;

    pub use crate::standard::Standard;

    pub use crate::terminate::{Terminated, Terminator};

    pub use crate::owned::SourceOwned;

    //pub use crate::debug::SourceDebug;
}
