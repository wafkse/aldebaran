#![doc = include_str!("../README.md")]
#![cfg_attr(not(test), no_std)]

pub mod brush;
pub mod predicate;
pub mod reset;
pub mod sequence;

pub mod prelude {
    //! The `aldebaran-ansi` prelude.
    //!
    //! Re-exports the most commonly used items from this crate.

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
