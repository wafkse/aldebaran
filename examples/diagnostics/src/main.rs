//! Multi-span diagnostic report construction and rendering.

use core::num::NonZero;

use aldebaran_report::{
    codegen::Report,
    prelude::{Fancy, FancySettings, InlineAnnotations, Label, Present, RenderMut, Report, Textual},
    render::textual::offload::fancy::{Colored, LayoutSettings, charset::Charset},
};
use aldebaran_span::prelude::Span;

#[derive(Debug, Report)]
#[error("duplicate binding")]
#[report(title = "duplicate binding", annotations = annotations)]
struct DuplicateBinding {
    annotations: InlineAnnotations<Label<&'static str>, 5>,
}

impl DuplicateBinding {
    fn new(current: Span, previous: Span, statement: Span, initializer: Span, shadowed_read: Span, later_use: Span) -> Self {
        let primary = Label::new("`résumé` is declared again here", current);
        let related = [
            Label::new("the first declaration is here", previous),
            Label::new("the redeclaration spans this statement", statement),
            Label::new("this initializer belongs to the redeclaration", initializer),
            Label::new("this read refers to the binding being shadowed", shadowed_read),
            Label::new("this later use would resolve through the redeclaration", later_use),
        ];
        let annotations = InlineAnnotations::new(primary, related);

        Self { annotations }
    }
}

fn span_of(source: &str, needle: &str) -> Span {
    let start = source.find(needle).expect("example source contains the requested fragment");
    let length = NonZero::new(needle.len()).expect("example fragment is nonempty");

    Span::new(start, length)
}

fn span_of_after(source: &str, needle: &str, after: usize) -> Span {
    let tail = source.get(after..).expect("search starts at a UTF-8 boundary");
    let relative = tail.find(needle).expect("example source contains the requested fragment");
    let length = NonZero::new(needle.len()).expect("example fragment is nonempty");

    Span::new(after + relative, length)
}

fn main() {
    let source = "let résumé = \"draft\";\nlet résumé = résumé + \" final\";\nemit(résumé);\n";
    let previous = span_of(source, "résumé");
    let current = span_of_after(source, "résumé", previous.end());
    let statement = span_of(source, "let résumé = résumé + \" final\";");
    let initializer = span_of(source, "résumé + \" final\"");
    let shadowed_read = span_of_after(source, "résumé", current.end());
    let later_use = span_of_after(source, "résumé", shadowed_read.end());
    let error = DuplicateBinding::new(current, previous, statement, initializer, shadowed_read, later_use);
    let attached = Report::attach(&error, source);

    let mut output = String::new();
    let mut renderer = Textual::<'_, _, Fancy, _>::new(&mut output);
    let settings = FancySettings::tuple((Colored::No, LayoutSettings::standard(), Charset::UNICODE));

    renderer
        .render_mut_with_input(Present::from_input(settings), &attached)
        .expect("diagnostic rendering succeeds");

    println!("{output}");
}
