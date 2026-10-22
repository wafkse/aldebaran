//! Text output composed from a writable sink and an [`Offload`] backend.
//!
//! [`Textual`] retains the sink between diagnostics. Each render receives a
//! [`Present`] value that combines backend input with a [`Viewport`], so callers
//! can change presentation policy without rebuilding the renderer.

pub mod offload;
pub mod viewport;

use core::fmt;
use core::hash;

use core::marker;

use offload::{Offload, OffloadContext};
use viewport::Viewport;

use crate::{render::RenderMut, report::SourceReport};

/// A backend's per-call input paired with a terminal-cell viewport.
///
/// Child backends can share the viewport and borrow
/// selected input through [`Self::referenced_with`].
// NOTE(invariant): None.
pub struct Present<'source, 'input, O, E>
where
    'source: 'input,
    O: Offload<'source, E> + ?Sized,
    E: SourceReport<'source>,
{
    /// Settings or resources interpreted by the selected backend.
    input: O::Input<'input>,

    /// Terminal viewport used to shape textual output.
    viewport: Viewport,
}

impl<'source, 'input, O, E> Copy for Present<'source, 'input, O, E>
where
    O: Offload<'source, E> + ?Sized,
    E: SourceReport<'source>,
    O::Input<'input>: Copy,
{
}

impl<'source, 'input, O, E> Clone for Present<'source, 'input, O, E>
where
    O: Offload<'source, E> + ?Sized,
    E: SourceReport<'source>,
    O::Input<'input>: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let &Self { ref input, viewport } = self;

        let input = input.clone();

        Self { input, viewport }
    }
}

impl<'source, 'input, O, E> Eq for Present<'source, 'input, O, E>
where
    O: Offload<'source, E> + ?Sized,
    E: SourceReport<'source>,
    O::Input<'input>: Eq,
{
}

impl<'source, 'input, O, E> PartialEq for Present<'source, 'input, O, E>
where
    O: Offload<'source, E> + ?Sized,
    E: SourceReport<'source>,
    O::Input<'input>: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.input() == other.input() && self.viewport() == other.viewport()
    }
}

impl<'source, 'input, O, E> hash::Hash for Present<'source, 'input, O, E>
where
    O: Offload<'source, E> + ?Sized,
    E: SourceReport<'source>,
    O::Input<'input>: hash::Hash,
{
    #[inline]
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
        self.input().hash(state);
        self.viewport().hash(state);
    }
}

impl<'source, 'input, O, E> fmt::Debug for Present<'source, 'input, O, E>
where
    O: Offload<'source, E> + ?Sized,
    E: SourceReport<'source>,
    O::Input<'input>: fmt::Debug,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Present")
            .field("input", self.input())
            .field("viewport", self.viewport())
            .finish()
    }
}

impl<'source, 'input, O, E> Default for Present<'source, 'input, O, E>
where
    O: Offload<'source, E> + ?Sized,
    E: SourceReport<'source>,
    O::Input<'input>: Default,
{
    #[inline]
    fn default() -> Self {
        let input = O::Input::<'input>::default();

        let viewport = Viewport::standard();

        Self { input, viewport }
    }
}

impl<'source, 'input, O, E> Present<'source, 'input, O, E>
where
    O: Offload<'source, E> + ?Sized,
    E: SourceReport<'source>,
{
    /// Use this backend input with the standard [`Viewport`].
    #[inline]
    pub const fn with_input(input: O::Input<'input>) -> Self {
        Self {
            input,
            viewport: Viewport::standard(),
        }
    }

    /// Use this [`Viewport`] with the backend's default input.
    #[inline]
    pub fn with_viewport(viewport: Viewport) -> Self
    where
        O::Input<'input>: Default,
    {
        let input = O::Input::<'input>::default();

        Self { input, viewport }
    }

    /// Combine explicit backend input and viewport without applying defaults.
    #[inline]
    pub const fn raw(input: O::Input<'input>, viewport: Viewport) -> Self {
        Self { input, viewport }
    }

    /// Settings or resources supplied to this backend for the render call.
    #[inline]
    pub const fn input(&self) -> &O::Input<'input> {
        let &Self { ref input, .. } = self;

        input
    }

    /// Terminal-cell budget and shape policy available to layout-aware backends.
    #[inline]
    pub const fn viewport(&self) -> &Viewport {
        let &Self { ref viewport, .. } = self;

        viewport
    }

    /// Create a presentation for a backend whose input borrows this backend's input.
    #[inline]
    pub fn referenced<'borrow, U>(&'borrow mut self) -> Present<'source, 'borrow, U, E>
    where
        'input: 'borrow,
        O::Input<'input>: 'borrow,
        U: Offload<'source, E, Input<'borrow> = &'borrow O::Input<'input>> + ?Sized,
    {
        self.referenced_with(move |input: &'borrow O::Input<'input>| input)
    }

    /// Project this backend's input for another backend.
    ///
    /// The returned presentation keeps the same viewport and borrows the
    /// projected input for `'borrow`.
    #[inline]
    pub fn referenced_with<'borrow, U, F, K>(&'borrow mut self, map: F) -> Present<'source, 'borrow, U, E>
    where
        'input: 'borrow,
        O::Input<'input>: 'borrow,
        U: Offload<'source, E, Input<'borrow> = K> + ?Sized,
        F: FnOnce(&'borrow O::Input<'input>) -> U::Input<'borrow>,
    {
        let &mut Self { ref input, viewport } = self;

        let input = map(input);

        Present { input, viewport }
    }
}

/// A mutable text renderer backed by [`Offload`].
///
/// Rendering appends to the sink. If formatting fails, output already written
/// remains and the backend error is returned.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
// NOTE(invariant): None.
pub struct Textual<'source, W, O, E>
where
    W: fmt::Write,
    O: Offload<'source, E> + ?Sized,
    E: SourceReport<'source>,
{
    /// The destination to write the report to.
    write_sink: W,

    /// Backend, diagnostic type, and source lifetime associated with this sink.
    _marker: (marker::PhantomData<O>, marker::PhantomData<E>, marker::PhantomData<&'source ()>),
}

impl<'source, W, O, E> RenderMut<'source, E> for Textual<'source, W, O, E>
where
    W: fmt::Write,
    O: Offload<'source, E> + ?Sized,
    E: SourceReport<'source>,
{
    type Input<'input>
        = Present<'source, 'input, O, E>
    where
        'source: 'input,
        E: 'source,
        Self: 'input;

    type Output = O::Output;

    type Error = O::Error;

    #[inline]
    fn render_mut_with_input<'input>(&'input mut self, input: Self::Input<'input>, error: &'source E) -> Result<Self::Output, Self::Error>
    where
        'source: 'input,
    {
        let &mut Self {
            write_sink: ref mut sink, ..
        } = self;

        let ctx = &mut OffloadContext::raw(sink, input);

        O::offload(error, ctx)
    }
}

impl<'source, W, O, E> Textual<'source, W, O, E>
where
    W: fmt::Write,
    O: Offload<'source, E> + ?Sized,
    E: SourceReport<'source>,
{
    /// Retain `write_sink` for subsequent renders, which append to its existing output.
    #[inline]
    pub const fn new(write_sink: W) -> Self {
        Self {
            write_sink,
            _marker: (marker::PhantomData, marker::PhantomData, marker::PhantomData),
        }
    }
}

// TODO: finish trait and impose it as a requirement for views instead of just
// bare print

#[cfg(test)]
mod tests {}
