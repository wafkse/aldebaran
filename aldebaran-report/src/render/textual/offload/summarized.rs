//! Compact textual diagnostics without source snippets.
//!
//! [`Summarized`] renders report identity and annotation messages in a short form
//! suitable for problem lists or logs. It intentionally omits source excerpts and
//! layout machinery while preserving structured annotation information.

use core::fmt;

use aldebaran_print::prelude::{Combine, Print};

use crate::annotated::Annotations;
use crate::{
    prelude::{Annotated, ErrorKind, Title},
    report::SourceReport,
};

use crate::render::textual::{Offload, OffloadContext};

/// A simplistic, summarized offload textual renderer.
///
/// This renders the error report in a summarized format, with the kind and
/// title, as well as the individual annotations.
///
/// This does not include any source snippets or additional context.
///
/// This is alike to what you would encounter inside your `problems` tab in your
/// IDE. It includes general error information, but no way to reference to them.
///
/// # Example
///
/// Albeit the actual format is not guaranteed to be stable across versions, it
/// will regardless follow a general style:
///
/// ```text
/// error: cannot find type `Summarized` in this scope
///     @ L:C  :: similarly named type `Summarizeda` found here
///     @ L:C  :: an enum with a similar name exists: `Summarizeda`
///
/// 2 extraneous annotations found
/// ```
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

        match annotations {
            Some(annotations) => {
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
            None => {}
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZero;

    use crate::{
        prelude::{Label, oneshot},
        render::{
            RenderMut,
            textual::{Present, Textual},
        },
    };

    use aldebaran_span::prelude::Span;

    use super::Summarized;

    #[test]
    fn test_fancy_offload() {
        let target_source: &'static str = "snicker";

        let err = oneshot::single(
            target_source,
            Label::new(
                "you hate snickers?? so do I!!",
                Span::new(0, NonZero::<usize>::MIN.saturating_add(10)),
            ),
        );

        dbg!(&err);

        let mut target_sink = String::new();

        let mut renderer = Textual::<'_, _, Summarized, _>::new(&mut target_sink);

        renderer
            .render_mut_with_input(Present::from_input(()), &err)
            .expect("failed to render error");

        println!("{}", target_sink);
    }
}
