pub mod macros {
    //! This module contains all the macros used in the crate.

    pub use aldebaran_primitive_macro::*;
}

pub mod prelude {
    //! A prelude for the `aldebaran-primitive` crate.
    //!
    //! This simply re-exports the most commonly used items in the
    //! `aldebaran-primitive` crate.

    pub use aldebaran_primitive_core::{Primitive, Signed, Unsigned};

    pub use aldebaran_primitive_core::cast::{Cast, LosslessCast, TryCast};

    pub use aldebaran_primitive_core::op::Operate;
}
