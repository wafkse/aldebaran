//! [`Nothing`] implements [`Offload`] while discarding the report, so callers can
//! suppress output without changing the rendering pipeline.

use core::convert::Infallible;

use core::fmt;

use crate::report::SourceReport;

use super::{Offload, OffloadContext};

/// Offload backend that deliberately produces no output.
///
/// This enum carries no state. Its [`Offload`] implementation lets callers
/// select suppression through the same interface as other backends.
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
