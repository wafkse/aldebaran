//! Styled source-aware textual diagnostics intended for human readers.
//!
//! [`Fancy`] composes header rendering, line segmentation, viewport layout,
//! annotation lanes, character sets, and theme policy. Source attachment remains
//! external through `SourceReport`, so this backend only borrows report inputs.

pub mod charset;
pub mod component;
pub mod layout;
pub mod setting;
pub mod theme;

pub use self::{
    component::{ByLines, Header},
    layout::LayoutSettings,
    setting::{Colored, FancySettings, Fancyness},
    theme::{DefaultTheme, Theme},
};

use core::{borrow::BorrowMut, fmt, marker};

use aldebaran_visualize::visual::Visualize;

use aldebaran_print::prelude::Print;

use aldebaran_span::span::Span;

use crate::{
    annotated::{Annotated, Annotations},
    prelude::{ErrorKind, Title},
    report::SourceReport,
};
use aldebaran_source::prelude::{LineSegmented, Location, Source, SourceLines, SourceMetadata};

use crate::render::textual::{Offload, OffloadContext};

/// A fancy, styled offload renderer for humans.
///
/// # Formatting Behavior
///
/// This renderer attempts to keep everything as per the [`FancySettings`]
/// provided on a best-effort basis. However, *some aspects* must be noted:
///
/// ## Inline Annotations and viewport extents
///
/// The renderer will constantly attempt to maintain the following goals:
///
/// - Each inline annotation will be rendered on the same line, without being cut off by the viewport \[*\].
///
/// - Lines will be wrapped at the viewport's width, and any excess text will be cut off and moved to the next line. This line wrapping has
///   no underlying logic: if it does not fit, it will be cut off.
///
/// ## Line Wrapping
///
/// This has no support for line wrapping, and thus will cut off any excess text
/// that does not fit inside the target viewport to the next line.
///
/// # Panics
///
/// This renderer is not expected to panic under any circumstances. If it does,
/// it is a bug.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Fancy<T = DefaultTheme>
where
    T: Theme,
{
    _marker: marker::PhantomData<T>,
}

impl<'source, E, T> Offload<'source, E> for Fancy<T>
where
    E: SourceReport<'source>,
    T: Theme + 'static,
    E::Source: SourceLines<'source> + SourceMetadata<'source> + 'source,
    <E::Source as SourceMetadata<'source>>::Metadata: Location,
    <E::Source as SourceLines<'source>>::Line: Print,
    <<E::Kind as ErrorKind>::Descriptor as Print>::Context: Default,
    <E::Title as Title>::Context: Default,
    <<E::Source as SourceMetadata<'source>>::Metadata as Print>::Context: Default,
    <<<E::Annotations as Annotations>::Annotation as Annotated>::Title as Title>::Context: Default,
    <<E::Source as SourceLines<'source>>::Line as LineSegmented<'source, E::Source>>::View: Visualize,
{
    type Input<'input>
        = FancySettings<T>
    where
        'source: 'input;

    type Output = ();

    type Error = fmt::Error;

    fn offload<'input, W>(error: &'source E, ctx: &mut OffloadContext<'source, 'input, W, Self, E>) -> Result<Self::Output, Self::Error>
    where
        'source: 'input,
        W: fmt::Write,
    {
        Header::offload(error, ctx.referenced().borrow_mut())?;

        let source = error.source();
        let segment = error.target().and_then(|target| {
            let source_size = source.size();

            match source.segment(target) {
                Some(extents) => Some(extents),
                None => match (target.start() == source_size, source_size) {
                    (true, 0) => None,
                    (true, _) => source.segment(Span::unit(source_size - 1)),
                    (false, _) => None,
                },
            }
        });

        match segment {
            Some(extents) => ByLines::offload(error, ctx.referenced_with(move |input| (input, extents)).borrow_mut()),
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZero;

    use crate::{
        prelude::{Label, oneshot},
        render::{
            RenderMut,
            textual::{
                Present, Textual,
                offload::fancy::{Colored, FancySettings, Fancyness, LayoutSettings, charset::Charset},
            },
        },
    };

    use aldebaran_span::prelude::Span;

    use super::Fancy;

    #[test]
    fn renders_source_boundary_span() {
        let source = "abc";
        let annotation = Label::new("missing input", Span::unit(source.len()));
        let report = oneshot::single(source, annotation);
        let mut output = String::new();
        let mut renderer = Textual::<'_, _, Fancy, _>::new(&mut output);
        let settings = Present::from_input(FancySettings::level(Fancyness::Outblown));

        renderer
            .render_mut_with_input(settings, &report)
            .expect("failed to render source boundary diagnostic");

        assert!(output.contains("missing input"));
    }

    #[test]
    fn unicode_source_marker_uses_rendered_width() {
        let source = "let résumé = 1;\n";
        let start = source.find("résumé").expect("unicode binding is present");
        let length = NonZero::new("résumé".len()).expect("unicode binding is nonempty");
        let annotation = Label::new("unicode binding", Span::new(start, length));
        let report = oneshot::single(source, annotation);
        let mut output = String::new();
        let mut renderer = Textual::<'_, _, Fancy, _>::new(&mut output);
        let settings = FancySettings::tuple((Colored::No, LayoutSettings::standard(), Charset::UNICODE));

        renderer
            .render_mut_with_input(Present::from_input(settings), &report)
            .expect("failed to render unicode diagnostic");

        assert!(output.contains("let résumé = 1;"));
        assert!(output.contains("│     ^^^^^^"));
        assert!(!output.contains("│     ^^^^^^^^"));
    }

    #[test]
    fn test_fancy_offload() {
        let target_source: &'static str = "snicker\n dog bone\n";

        let err = oneshot::single(
            target_source,
            Label::new("you hate snickers?? so do I!!", Span::new(0, const { NonZero::<usize>::MAX })),
        );

        dbg!(&err);

        let mut target_sink = String::new();

        let mut renderer = Textual::<'_, _, Fancy, _>::new(&mut target_sink);

        renderer
            .render_mut_with_input(Present::from_input(FancySettings::level(Fancyness::Outblown)), &err)
            .expect("failed to render error");

        println!("{}", target_sink);
    }
}
