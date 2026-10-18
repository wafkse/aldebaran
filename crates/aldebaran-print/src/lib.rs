#![no_std]
#![doc = include_str!("../README.md")]

pub mod combine;
pub mod print;
pub mod style;
pub mod visualize;

pub mod prelude {
    //! The `aldebaran-print` prelude.
    //!
    //! Re-exports the most commonly used items from this crate.

    pub use crate::combine::Combine;
    pub use crate::print::Print;
    pub use crate::style::Stylus;
    pub use crate::visualize::Visualize;
}
