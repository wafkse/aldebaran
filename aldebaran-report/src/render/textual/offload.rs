//! Offload renderers for textual reports.
//!
//! This module contains the [`Offload`] trait.
//! trait that allows for "offloading" the rendering process to an arbitrary,
//! user-defined renderer.
//!
//! This is the way to go for when a custom renderer is needed.
//!
//! See [`Offload`] for further guidance.

pub mod fancy;
pub mod hexdump;
pub mod nothing;
pub mod summarized;

use core::fmt;
use core::hash;

use crate::report::SourceReport;

use crate::render::textual::Present;

/// A trait for offloading the rendering of a textual report to a different
/// renderer.
///
/// This trait is identical to both [`Render`] and [`RenderMut`], but has no
/// dispatcher type for its primary method.
///
/// [`Render`]: crate::render::Render
/// [`RenderMut`]: crate::render::RenderMut
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

/// An offload context for subordinates to render a textual report.
///
/// This structure contains the sink to write the report to and the
/// presentational settings.
///
/// See [`Offload`] for more information.
pub struct OffloadContext<'source, 'input, W, O, E>
where
    'source: 'input,
    W: fmt::Write,
    O: Offload<'source, E> + ?Sized,
    E: SourceReport<'source>,
{
    sink: W,
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
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
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
        self.sink == other.sink && self.present == other.present
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
    /// Create a new offload context from its raw parts: the sink and the
    /// presentational settings.
    ///
    /// Not part of public API.
    #[inline]
    pub(super) const fn from_raw_parts(sink: W, present: Present<'source, 'input, O, E>) -> Self {
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

    /// Borrow the components of this offload context in a single atomic
    /// operation.
    #[inline]
    pub const fn decompose(&mut self) -> (&mut W, &Present<'source, 'input, O, E>) {
        let &mut Self { ref mut sink, ref present } = self;

        (sink, present)
    }

    /// Map in a new [`OffloadContext`] whose [`Offload::Input`] is directly borrowed from `O::Input`.
    ///
    /// This is equivalent to [`Self::referenced_with`] when using [`core::convert::identity`] as the target function.
    #[inline]
    pub fn referenced<'borrow, U>(&'borrow mut self) -> OffloadContext<'source, 'borrow, &'borrow mut W, U, E>
    where
        'input: 'borrow,
        O::Input<'input>: 'borrow,
        U: Offload<'source, E, Input<'borrow> = &'borrow O::Input<'input>> + ?Sized,
    {
        self.referenced_with(core::convert::identity)
    }

    /// Map in a new [`OffloadContext`] whose [`Offload::Input`] can be directly sourced from the current [`O::Input`] using an
    /// user-supplied function `F`.
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
