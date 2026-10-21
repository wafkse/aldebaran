//! Diagnostic headings styled independently of source excerpts.
//!
//! [`Header`] applies the selected theme to a report's kind and title. It shares
//! [`FancySettings`] with the source renderer so both stages follow the same
//! color policy.

use core::{fmt, marker};

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

/// A diagnostic heading styled from kind metadata under [`FancySettings`].
///
/// The kind descriptor and title use their default print contexts, and a
/// formatting error can leave a partial heading in the destination.
#[derive(Debug, Copy, Hash, Clone)]
// NOTE(invariant): None.
pub struct Header<T = DefaultTheme>(marker::PhantomData<T>)
where
    T: Theme;

impl<'source, E, T> Offload<'source, E> for Header<T>
where
    E: SourceReport<'source>,
    T: Theme,
    <<E::Kind as ErrorKind>::Descriptor as Print>::Context: Default,
    <E::Title as Title>::Context: Default,
{
    type Input<'input>
        = &'input FancySettings<T>
    where
        'source: 'input;

    type Output = ();

    type Error = fmt::Error;

    #[inline]
    fn offload<'input, W>(error: &'source E, ctx: &mut OffloadContext<'source, 'input, W, Self, E>) -> Result<Self::Output, Self::Error>
    where
        'source: 'input,
        W: fmt::Write,
    {
        let present = ctx.present();

        let &settings = present.input();

        let is_colored = &Should::tuple((settings.colored_ref(), Colored::colored));

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
