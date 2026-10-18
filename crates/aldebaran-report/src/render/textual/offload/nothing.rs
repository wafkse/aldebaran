//! Output suppression as a textual backend policy.

use core::convert::Infallible;

use core::fmt;

use crate::report::SourceReport;

use super::{Offload, OffloadContext};

/// Uninhabited backend that returns success without writing to the sink.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Nothing {}

impl<'source, E> Offload<'source, E> for Nothing
where
    E: SourceReport<'source>,
{
    type Input<'input>
        = ()
    where
        'source: 'input;

    type Output = ();

    type Error = Infallible;

    #[inline]
    fn offload<'input, W>(_: &'source E, _: &mut OffloadContext<'source, 'input, W, Self, E>) -> Result<Self::Output, Self::Error>
    where
        'source: 'input,
        W: fmt::Write,
    {
        Ok(())
    }
}
