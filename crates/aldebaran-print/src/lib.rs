#![no_std]
#![doc = include_str!("../../../docs/part/COMPONENT.md")]
#![doc = include_str!("../README.md")]
#![doc = include_str!("../../../docs/part/FACADE.md")]
#![doc = include_str!("../../../docs/part/LICENSE.md")]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
pub mod layout;

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
