//! Formatting backends used by the [`Textual`](super::Textual) renderer.
//!
//! See [`Offload`] and [`OffloadContext`] for further information.

pub mod fancy;
pub mod hexdump;
pub mod nothing;
pub mod summarized;

use core::convert;
use core::fmt;
use core::hash;

use crate::report::SourceReport;

use crate::render::textual::Present;

/// A backend that formats a textual report through an [`OffloadContext`].
///
/// Unlike [`Render`] and [`RenderMut`], its method does not take a renderer
/// value. The context supplies the destination and presentation settings.
///
/// [`Render`]: crate::render::Render
/// [`RenderMut`]: crate::render::RenderMut
///
/// Each backend defines its input, output, and errors, and decides whether a
/// failure can leave partial output in the context's sink.
pub trait Offload<'source, E>
where
    E: SourceReport<'source>,
{
    /// The input type for the offload function.
    type Input<'input>
    where
        'source: 'input;

    /// The output type for the offload function.
    type Output;

    /// The error type for the offload function.
    type Error;

    /// Offload the rendering of a textual report to this offload renderer.
    fn offload<'input, W>(error: &'source E, ctx: &mut OffloadContext<'source, 'input, W, Self, E>) -> Result<Self::Output, Self::Error>
    where
        'source: 'input,
        W: fmt::Write;
}

/// The sink and presentation settings an [`Offload`] backend uses to format a
/// report.
///
/// Child backends can borrow the same sink and input through [`Self::referenced`],
/// or project that input through [`Self::referenced_with`].
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
    #[inline]
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
    #[inline]
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
    pub(in crate::render::textual) const fn raw(sink: W, present: Present<'source, 'input, O, E>) -> Self {
        Self { sink, present }
    }

    /// Retrieve the [`fmt::Write`] sink for this offload context.
    ///
    /// This sink is used to write the report to.
    #[inline]
    pub const fn sink(&mut self) -> &mut W {
        let &mut Self { ref mut sink, .. } = self;

        sink
    }

    /// Retrieve the presentational settings for this offload context.
    ///
    /// See [`Present`] for more information.
    #[inline]
    pub const fn present(&self) -> &Present<'source, 'input, O, E> {
        let &Self { ref present, .. } = self;

        present
    }

    /// Borrow the destination and presentation settings together.
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

        OffloadContext::raw(sink, present)
    }
}
