//! ANSI terminal styling primitives for printable values.
//!
//! The crate models colors, text attributes, escape sequences, and deferred style
//! application independently. Consumers can compose presentation without forcing
//! ANSI concerns into the semantic values being printed.

#![cfg_attr(not(test), no_std)]
//! ANSI colors, text attributes, and deferred terminal painting for
//! `aldebaran-print` values.

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
    fn color() {
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
