//! A compact report for logs and problem lists that do not need source excerpts.
//!
//! See [`Summarized`] for further information.

use core::fmt;

use aldebaran_print::prelude::{Combine, Print};

use crate::annotated::Annotations;
use crate::{
    prelude::{Annotated, ErrorKind, Title},
    report::SourceReport,
};

use crate::render::textual::{Offload, OffloadContext};

/// Renders the report's kind and title, followed by each annotation's span and message.
///
/// It omits source excerpts, so annotated locations remain visible without
/// printing source text. When annotations are present, the final count gives
/// the number that follow the primary annotation.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Summarized {}

impl<'source, E> Offload<'source, E> for Summarized
where
    E: SourceReport<'source>,
    <<E::Kind as ErrorKind>::Descriptor as Print>::Context: Default,
    <E::Title as Title>::Context: Default,
    <<<E::Annotations as Annotations>::Annotation as Annotated>::Title as Title>::Context: Default,
{
    type Input<'input>
        = ()
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
        let sink = ctx.sink();

        let kind = error.kind();
        let title = error.title();

        kind.descriptor().print(sink)?;

        ':'.sequence(' ').sequence(title.content()).sequence('\n').print(sink)?;

        let annotations = error.annotations().list();

        if let Some(annotations) = annotations {
            for annotation in annotations.iter() {
                let message = annotation.message();
                let target = annotation.target();

                '\t'.sequence('@')
                    .sequence(target)
                    .sequence(' ')
                    .sequence(':'.times(2))
                    .sequence(' ')
                    .sequence(message.content())
                    .sequence('\n')
                    .print(sink)?
            }

            '\n'.sequence(annotations.rest().len())
                .sequence(' ')
                .sequence("extraneous annotations found")
                .sequence('\n')
                .print(sink)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    //! Tests module for the [`summarized`] module.
    //!
    //! [`summarized`]: super

    use core::num::NonZero;

    use crate::{
        prelude::{Label, oneshot},
        render::{
            RenderMut,
            textual::{Present, Textual},
        },
    };

    use aldebaran_baseline::prelude::Span;

    use super::Summarized;

    #[test]
    fn fancy_offload() {
        let target_source = "snicker";

        let err = oneshot::single(
            target_source,
            Label::new(
                "you hate snickers?? so do I!!",
                Span::new(0, NonZero::<usize>::MIN.saturating_add(10)),
            ),
        );

        let mut target_sink = String::new();

        let mut renderer = Textual::<'_, _, Summarized, _>::new(&mut target_sink);

        renderer
            .render_mut_with_input(Present::with_input(()), &err)
            .expect("failed to render error");

        println!("{}", target_sink);
    }
}
