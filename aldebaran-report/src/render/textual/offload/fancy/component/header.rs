//! Header rendering for fancy textual reports.
//!
//! [`Header`] emits the report kind, title, and source location before line-based
//! snippets are rendered. Styling comes from the active theme and presentation
//! settings, keeping header layout independent from concrete diagnostic types.

use core::{fmt, hash};

use aldebaran_ansi::{brush::Paintable, predicate::Should};
use aldebaran_print::prelude::{Combine, Print};

use crate::{
    prelude::{ErrorKind, Title},
    render::textual::{
        offload::fancy::{Colored, DefaultTheme, FancySettings, Theme},
        offload::{Offload, OffloadContext},
    },
    report::{SourceReport, connotate::Connotate},
};

/// Rendering stage for the leading report header.
///
/// The stage projects report metadata and source location through the selected
/// theme, then writes the stable header structure before source annotations begin.
pub struct Header<T = DefaultTheme>
where
    T: Theme,
{
    _marker: core::marker::PhantomData<T>,
}

impl<T> Copy for Header<T> where T: Theme {}

impl<T> Clone for Header<T>
where
    T: Theme,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            _marker: core::marker::PhantomData,
        }
    }
}

impl<T> hash::Hash for Header<T>
where
    T: Theme,
{
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { _marker, .. } = self;

        _marker.hash(state);
    }
}

impl<T> fmt::Debug for Header<T>
where
    T: Theme,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Header").field("_marker", &self._marker).finish()
    }
}

impl<'source, E, T> Offload<'source, E> for Header<T>
where
    E: SourceReport<'source>,
    T: Theme + 'static,
    <<E::Kind as ErrorKind>::Descriptor as Print>::Context: Default,
    <E::Title as Title>::Context: Default,
{
    type Input<'input>
        = &'input FancySettings<T>
    where
        'source: 'input;

    type Output = ();

    type Error = fmt::Error;

    fn offload<'input, W>(error: &'source E, ctx: &mut OffloadContext<'source, 'input, W, Self, E>) -> Result<Self::Output, Self::Error>
    where
        'source: 'input,
        W: fmt::Write,
    {
        let present = ctx.present();

        let &settings = present.input();

        let ref is_colored = Should::tuple((settings.colored_ref(), Colored::colored));

        let kind = error.kind();

        let theme = settings.theme().metadata(&kind.connotates());

        let title = error.title();

        kind.descriptor()
            .stylable()
            .styled(theme.kind())
            .only_when(is_colored)
            .sequence(':')
            .sequence(' ')
            .sequence(title.content().stylable().styled(theme.title()).only_when(is_colored))
            .sequence('\n')
            .print(ctx.sink())
    }
}
