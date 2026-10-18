//! Generic predicates that do not depend on a particular input domain.
//!
//! These predicates implement the logic contracts directly and can be composed
//! with the crate's connective types.

use core::fmt;

use crate::{
    assert::Assert,
    fmt::{Formatter, Precedence},
    input::Input,
};

use aldebaran_print::print::Print;

/// An assertion that accepts every input value.
#[derive(Debug, Clone, Copy, Hash, Default)]
pub struct Anything;

impl<I> Assert<I> for Anything
where
    I: Input,
{
    #[inline]
    fn assert(&self, _: I) -> bool {
        true
    }

    #[inline]
    fn output_with<W, F>(target_value: &Self, writer: &mut W, _: Precedence) -> fmt::Result
    where
        W: fmt::Write,
        F: Formatter,
    {
        Self::print(target_value, writer)
    }
}

impl Print for Anything {
    type Context = ();

    #[inline]
    fn print_with_context<W>(&self, writer: &mut W, _: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        writer.write_str("<anything>")
    }
}
