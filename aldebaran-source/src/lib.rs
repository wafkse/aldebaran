//! Generic source capabilities and line-oriented source metadata.
//!
//! The crate models source data through independent iteration, dissection,
//! metadata, hashing, location, and line capabilities. Concrete byte and string
//! sources implement these mechanisms without forcing one universal source type,
//! while line handles preserve absolute spans into the original source.

#![no_std]
#![forbid(rustdoc::all, clippy::all)]

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
    //! A prelude for the `aldebaran-source` crate.
    //!
    //! This simply re-exports the most commonly used items in the
    //! `aldebaran-source` crate.

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
