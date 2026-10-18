//! Sink backend that intentionally emits no diagnostic output.
//!
//! [`Nothing`] satisfies the textual offload interface while discarding the
//! report. It is useful when callers need to preserve the rendering pipeline but
//! select suppression as an explicit backend policy.

use core::convert::Infallible;

use core::fmt;

use crate::report::SourceReport;

use super::{Offload, OffloadContext};

/// Offload backend that deliberately produces no output.
///
/// The empty enum carries no state. Its implementation exists only to represent
/// suppression as a normal renderer choice rather than a special caller branch.
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
