//! Formatting policies used by the [`Textual`](super::Textual) renderer.
//!
//! [`Offload`] defines backend input and output, while [`OffloadContext`] makes
//! the sink and presentation available during a render call. Composed backends
//! can borrow the same sink and project input for a child backend without
//! transferring ownership of the destination.

pub mod fancy;
pub mod hexdump;
pub mod nothing;
pub mod summarized;

use core::convert;
use core::fmt;
use core::hash;

use crate::report::SourceReport;

use crate::render::textual::Present;

/// A formatting policy selected independently of the destination sink.
///
/// Implementations define the required input and partial-output behavior, while
/// [`OffloadContext`] provides the destination sink and viewport.
pub trait Offload<'source, E>
where
    E: SourceReport<'source>,
{
    /// Settings or borrowed resources needed for this backend's presentation policy.
    type Input<'input>
    where
        'source: 'input;

    /// Completion value, often `()` when the sink receives the rendered text.
    type Output;

    /// Backend-defined failures, including any formatting errors it propagates.
    type Error;

    /// Write the diagnostic under the context's presentation policy.
    fn offload<'input, W>(error: &'source E, ctx: &mut OffloadContext<'source, 'input, W, Self, E>) -> Result<Self::Output, Self::Error>
    where
        'source: 'input,
        W: fmt::Write;
}

/// The output destination and presentation passed to an [`Offload`] backend.
///
/// Child backends can reuse the destination with the current input through
/// [`Self::referenced`], or with a projected input through
/// [`Self::referenced_with`].
// NOTE(invariant): None.
pub struct OffloadContext<'source, 'input, W, O, E>
where
    'source: 'input,
    W: fmt::Write,
    O: Offload<'source, E> + ?Sized,
    E: SourceReport<'source>,
{
    /// Destination receiving rendered text.
    sink: W,

    /// Presentation state supplied to the active offload renderer.
    present: Present<'source, 'input, O, E>,
}

impl<'source, 'input, W, O, E> Default for OffloadContext<'source, 'input, W, O, E>
where
    W: fmt::Write,
    O: Offload<'source, E> + ?Sized,
    E: SourceReport<'source>,
    W: Default,
    O::Input<'input>: Default,
{
    #[inline]
    fn default() -> Self {
        Self {
            sink: Default::default(),
            present: Default::default(),
        }
    }
}

impl<'source, 'input, W, O, E> Clone for OffloadContext<'source, 'input, W, O, E>
where
    'source: 'input,
    W: fmt::Write,
    O: Offload<'source, E> + ?Sized,
    E: SourceReport<'source>,
    W: Clone,
    O::Input<'input>: Clone,
    E: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { sink, present } = self;

        let sink = sink.clone();
        let present = present.clone();

        Self { sink, present }
    }
}

impl<'source, 'input, W, O, E> hash::Hash for OffloadContext<'source, 'input, W, O, E>
where
    'source: 'input,
    W: fmt::Write,
    O: Offload<'source, E> + ?Sized,
    E: SourceReport<'source>,
    W: hash::Hash,
    O::Input<'input>: hash::Hash,
{
    #[inline]
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
        let Self { sink, present } = self;

        sink.hash(state);
        present.hash(state);
    }
}

impl<'source, 'input, W, O, E> Eq for OffloadContext<'source, 'input, W, O, E>
where
    'source: 'input,
    W: fmt::Write,
    O: Offload<'source, E> + ?Sized,
    E: SourceReport<'source>,
    W: Eq,
    O::Input<'input>: Eq,
{
}

impl<'source, 'input, W, O, E> PartialEq for OffloadContext<'source, 'input, W, O, E>
where
    'source: 'input,
    W: fmt::Write,
    O: Offload<'source, E> + ?Sized,
    E: SourceReport<'source>,
    W: PartialEq,
    O::Input<'input>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { sink, present } = self;

        let Self {
            sink: other_sink,
            present: other_present,
        } = other;

        sink == other_sink && present == other_present
    }
}

impl<'source, 'input, W, O, E> fmt::Debug for OffloadContext<'source, 'input, W, O, E>
where
    'source: 'input,
    W: fmt::Write,
    O: Offload<'source, E> + ?Sized,
    E: SourceReport<'source>,
    W: fmt::Debug,
    O::Input<'input>: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { sink, present } = self;

        f.debug_struct("OffloadContext")
            .field("sink", sink)
            .field("present", present)
            .finish()
    }
}

impl<'source, 'input, W, O, E> OffloadContext<'source, 'input, W, O, E>
where
    'source: 'input,
    W: fmt::Write,
    O: Offload<'source, E> + ?Sized,
    E: SourceReport<'source>,
{
    /// Pair an output sink with presentation settings.
    #[inline]
    pub(in crate::render::textual) const fn from_raw_parts(sink: W, present: Present<'source, 'input, O, E>) -> Self {
        Self { sink, present }
    }

    /// Destination shared by this backend and any child backends.
    #[inline]
    pub const fn sink(&mut self) -> &mut W {
        let &mut Self { ref mut sink, .. } = self;

        sink
    }

    /// Backend settings and viewport selected for the current diagnostic.
    #[inline]
    pub const fn present(&self) -> &Present<'source, 'input, O, E> {
        let &Self { ref present, .. } = self;

        present
    }

    /// Access the destination while retaining shared access to presentation policy.
    #[inline]
    pub const fn decompose(&mut self) -> (&mut W, &Present<'source, 'input, O, E>) {
        let &mut Self { ref mut sink, ref present } = self;

        (sink, present)
    }

    /// Create a child context that reuses this backend's input and sink.
    #[inline]
    pub fn referenced<'borrow, U>(&'borrow mut self) -> OffloadContext<'source, 'borrow, &'borrow mut W, U, E>
    where
        'input: 'borrow,
        O::Input<'input>: 'borrow,
        U: Offload<'source, E, Input<'borrow> = &'borrow O::Input<'input>> + ?Sized,
    {
        self.referenced_with(convert::identity)
    }

    /// Project this backend's input into a child context that reuses the sink.
    ///
    /// [`O::Input`]: Offload::Input
    #[inline]
    pub fn referenced_with<'borrow, U, F, K>(&'borrow mut self, map: F) -> OffloadContext<'source, 'borrow, &'borrow mut W, U, E>
    where
        'input: 'borrow,
        O::Input<'input>: 'borrow,
        U: Offload<'source, E, Input<'borrow> = K> + ?Sized,
        F: FnOnce(&'borrow O::Input<'input>) -> U::Input<'borrow>,
    {
        let &mut Self {
            ref mut sink,
            ref mut present,
        } = self;

        let present = present.referenced_with(map);

        OffloadContext::from_raw_parts(sink, present)
    }
}
