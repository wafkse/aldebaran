//! Rendering contracts for diagnostics paired with their source.
//!
//! [`Render`] permits shared access to renderer state, while [`RenderMut`] also
//! supports retained writable sinks. The [`textual`] adapter combines such a
//! sink with a formatting backend and per-call presentation settings.

use crate::report::SourceReport;

pub mod textual;

/// A renderer that produces output through shared access to its state.
///
/// Use [`RenderMut`] when rendering needs to update the renderer, such as when
/// it retains a writable sink.
pub trait Render<'source, E>
where
    E: SourceReport<'source>,
{
    /// Per-call presentation data, which may borrow the renderer or source.
    type Input<'input>
    where
        'source: 'input,
        E: 'source,
        Self: 'input;

    /// Completion value, which may be `()` when rendering writes to a retained sink.
    type Output;

    /// Renderer-defined failures, including any output errors it propagates.
    type Error;

    /// Produce diagnostic output under the supplied presentation policy.
    fn render_with_input<'input>(
        &'input self,
        input: <Self as Render<'source, E>>::Input<'input>,
        error: &'source E,
    ) -> Result<<Self as Render<'source, E>>::Output, <Self as Render<'source, E>>::Error>
    where
        'source: 'input;

    /// Use the input type's default presentation policy.
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

/// A renderer that can retain writable state between diagnostics.
///
/// Implementations define whether errors leave partial output or retained state changes.
pub trait RenderMut<'source, E>
where
    E: SourceReport<'source>,
{
    /// Per-call presentation data, which may borrow the renderer or source.
    type Input<'input>
    where
        'source: 'input,
        E: 'source,
        Self: 'input;

    /// Completion value, which may be `()` when rendering writes to a retained sink.
    type Output;

    /// Renderer-defined failures, including any output errors it propagates.
    type Error;

    /// Produce diagnostic output using the renderer's writable state.
    fn render_mut_with_input<'input>(&'input mut self, input: Self::Input<'input>, error: &'source E) -> Result<Self::Output, Self::Error>
    where
        'source: 'input;

    /// Use the input type's default presentation policy.
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
