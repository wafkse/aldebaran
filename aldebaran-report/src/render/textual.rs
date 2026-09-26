//! This module revolves around the [`Textual`] [`Render`], that is, an engine
//! that makes creating human-readable error reports possible.
//!
//! [`Render`]: super::Render

pub mod offload;
pub mod viewport;

use core::fmt;
use core::hash;

use core::marker;

use offload::{Offload, OffloadContext};
use viewport::Viewport;

use crate::{render::RenderMut, report::SourceReport};

/// Presentational settings for the textual renderer.
///
/// This structure contains the settings that are used to render the error
/// report
pub struct Present<'source, 'input, O, E>
where
    'source: 'input,
    O: Offload<'source, E> + ?Sized,
    E: SourceReport<'source>,
{
    /// Renderer-specific presentation input.
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
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
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
    /// Create a new set of presentational settings from solely the input, using
    /// a standard viewport.
    ///
    /// This is the same as calling [`Present::from_raw_parts`] with a standard
    /// viewport retrieved through [`Viewport::standard()`].
    #[inline]
    pub const fn from_input(input: O::Input<'input>) -> Self {
        Self {
            input,
            viewport: Viewport::standard(),
        }
    }

    /// Create a new set of presentational settings from solely the viewport,
    ///
    /// Uses the default input for the offload renderer.
    #[inline]
    pub fn from_viewport(viewport: Viewport) -> Self
    where
        O::Input<'input>: Default,
    {
        let input = O::Input::<'input>::default();

        Self { input, viewport }
    }

    /// Create a new set of presentational settings from its raw parts.
    #[inline]
    pub const fn from_raw_parts(input: O::Input<'input>, viewport: Viewport) -> Self {
        Self { input, viewport }
    }

    /// Retrieve a reference the [`offload renderer input`](Offload::Input)
    /// associated with these presentational settings.
    #[inline]
    pub const fn input(&self) -> &O::Input<'input> {
        let &Self { ref input, .. } = self;

        input
    }

    /// Retrieve a reference to the [`Viewport`] associated with these
    /// presentational settings.
    #[inline]
    pub const fn viewport(&self) -> &Viewport {
        let &Self { ref viewport, .. } = self;

        viewport
    }

    /// Reinstate a different offload context for a distinct offload renderer
    /// that has an input type borrowing the input type of the current offload
    /// renderer.
    #[inline]
    pub fn referenced<'borrow, U>(&'borrow mut self) -> Present<'source, 'borrow, U, E>
    where
        'input: 'borrow,
        O::Input<'input>: 'borrow,
        U: Offload<'source, E, Input<'borrow> = &'borrow O::Input<'input>> + ?Sized,
    {
        self.referenced_with(move |input: &'borrow O::Input<'input>| input)
    }

    /// An input-mapping version of [`referenced`].
    ///
    /// [`referenced`]: Present::referenced
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

/// A textual renderer that can be used to create human-readable error reports.
///
/// See [`Render`] and [`RenderMut`] for more information.
///
/// [`Render`]: super::Render
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
pub struct Textual<'source, W, O, E>
where
    W: fmt::Write,
    O: Offload<'source, E> + ?Sized,
    E: SourceReport<'source>,
{
    /// The destination to write the report to.
    write_sink: W,

    /// A marker to indicate the offload renderer and error type on an
    /// implementation basis.
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

        let ref mut ctx = OffloadContext::from_raw_parts(sink, input);

        O::offload(error, ctx)
    }
}

impl<'source, W, O, E> Textual<'source, W, O, E>
where
    W: fmt::Write,
    O: Offload<'source, E> + ?Sized,
    E: SourceReport<'source>,
{
    /// Create a new [`Textual`] renderer that writes to the target [`Write`]r.
    ///
    /// [`Write`]: fmt::Write
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
