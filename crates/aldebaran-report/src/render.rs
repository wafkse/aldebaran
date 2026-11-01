//! This module contains the traits and adapters that turn source reports into
//! human-readable text or other structured output.

use crate::report::SourceReport;

pub mod textual;

/// A trait for types that can be used as a renderer for errors.
///
/// A renderer is a type that can take an input and produce an output, possibly
/// failing in the process.
///
/// See [`RenderMut`] if you also need to mutate the renderer itself.
pub trait Render<'source, E>
where
    E: SourceReport<'source>,
{
    /// Each render call receives this input, which may borrow the renderer or source.
    type Input<'input>
    where
        'source: 'input,
        E: 'source,
        Self: 'input;

    /// The value returned after rendering, which may be `()` when output is
    /// written to a retained sink.
    type Output;

    /// This type covers renderer failures and any output errors the renderer propagates.
    type Error;

    /// Render the target [`SourceReport`] into the target output, using the provided
    /// [`Self::Input`].
    fn render_with_input<'input>(
        &'input self,
        input: <Self as Render<'source, E>>::Input<'input>,
        error: &'source E,
    ) -> Result<<Self as Render<'source, E>>::Output, <Self as Render<'source, E>>::Error>
    where
        'source: 'input;

    /// Render the target [`SourceReport`] into the target output, using the
    /// [`default`](Default::default) value of [`Self::Input`].
    #[inline]
    fn render<'input>(&'input self, error: &'source E) -> Result<<Self as Render<'source, E>>::Output, <Self as Render<'source, E>>::Error>
    where
        'source: 'input,
        <Self as Render<'source, E>>::Input<'input>: Default,
    {
        let input: <Self as Render<'source, E>>::Input<'input> = Default::default();

        self.render_with_input(input, error)
    }
}

impl<'source, E, T> Render<'source, E> for &T
where
    E: SourceReport<'source>,
    T: Render<'source, E>,
{
    type Input<'input>
        = <T as Render<'source, E>>::Input<'input>
    where
        'source: 'input,
        E: 'source,
        Self: 'input;

    type Output = <T as Render<'source, E>>::Output;

    type Error = <T as Render<'source, E>>::Error;

    #[inline]
    fn render_with_input<'input>(
        &'input self,
        input: <Self as Render<'source, E>>::Input<'input>,
        error: &'source E,
    ) -> Result<<Self as Render<'source, E>>::Output, <Self as Render<'source, E>>::Error>
    where
        'source: 'input,
    {
        T::render_with_input(self, input, error)
    }
}

/// A trait for types that can be used as a renderer for errors.
///
/// A renderer is a type that can take an input and produce an output, possibly
/// failing in the process.
///
/// Implementations define whether errors leave partial output or retained state changes.
pub trait RenderMut<'source, E>
where
    E: SourceReport<'source>,
{
    /// Each render call receives this input, which may borrow the renderer or source.
    type Input<'input>
    where
        'source: 'input,
        E: 'source,
        Self: 'input;

    /// The value returned after rendering, which may be `()` when output is
    /// written to a retained sink.
    type Output;

    /// This type covers renderer failures and any output errors the renderer propagates.
    type Error;

    /// Render the target [`SourceReport`] into the target output, using the provided
    /// [`Self::Input`].
    fn render_mut_with_input<'input>(&'input mut self, input: Self::Input<'input>, error: &'source E) -> Result<Self::Output, Self::Error>
    where
        'source: 'input;

    /// Render the target [`SourceReport`] into the target output, using the
    /// [`default`](Default::default) value of [`Self::Input`].
    #[inline]
    fn render_mut<'input>(&'input mut self, error: &'source E) -> Result<Self::Output, Self::Error>
    where
        'source: 'input,
        Self::Input<'input>: Default,
    {
        let input: Self::Input<'input> = Default::default();

        self.render_mut_with_input(input, error)
    }
}

impl<'source, E, T> RenderMut<'source, E> for &mut T
where
    E: SourceReport<'source>,
    T: RenderMut<'source, E>,
{
    type Input<'input>
        = <T as RenderMut<'source, E>>::Input<'input>
    where
        'source: 'input,
        E: 'source,
        Self: 'input;

    type Output = <T as RenderMut<'source, E>>::Output;

    type Error = <T as RenderMut<'source, E>>::Error;

    #[inline]
    fn render_mut_with_input<'input>(
        &'input mut self,
        input: <Self as RenderMut<'source, E>>::Input<'input>,
        error: &'source E,
    ) -> Result<<Self as RenderMut<'source, E>>::Output, <Self as RenderMut<'source, E>>::Error>
    where
        'source: 'input,
    {
        T::render_mut_with_input(self, input, error)
    }
}
