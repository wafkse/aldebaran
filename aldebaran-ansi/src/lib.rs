//! ANSI terminal styling primitives for printable values.
//!
//! The crate models colors, text attributes, escape sequences, and deferred style
//! application independently. Consumers can compose presentation without forcing
//! ANSI concerns into the semantic values being printed.

#![cfg_attr(not(test), no_std)]
#![forbid(
    unsafe_code,
    missing_docs,
    missing_debug_implementations,
    clippy::missing_panics_doc,
    clippy::missing_safety_doc,
    clippy::missing_transmute_annotations,
    clippy::unwrap_used
)]
//! An ANSI terminal color painting library. Part of the `aldebaran-style`
//! subsystem.
//!
//! This model allows for exhaustive separation of concerns, and makes it
//! trivial to introduce new styles and colors.
//!
//! For more information regarding the usage of this library, see the
//! [`Paintable`](crate::prelude::Paintable) trait.

pub mod brush;
pub mod predicate;
pub mod reset;
pub mod sequence;

pub mod prelude {
    //! A prelude for the `aldebaran-ansi` crate.
    //!
    //! This simply re-exports the most commonly used items in the
    //! `aldebaran-ansi` crate.

    pub use crate::brush::{
        Paintable,
        color::{Color, Surface},
        style::Style,
    };

    pub use crate::predicate::ContextualPredicate;

    pub use crate::sequence::AnsiSequence;

    pub use crate::reset::{Reset, Resettable};
}

#[cfg(test)]
mod tests {
    use aldebaran_print::prelude::{Combine, Print};

    use crate::predicate::Should;
    pub use crate::prelude::Paintable;

    #[test]
    fn test_color() {
        let mut s = String::new();

        "fake and gay"
            .black()
            .on_red()
            .only_when(&Should::tuple((&true, |_| true)))
            .sequence('\n')
            .sequence("niogga!".yellow())
            .print(&mut s)
            .expect("failed to print");

        eprintln!("{}", s);
    }
}
