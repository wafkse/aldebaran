use aldebaran_report::{
    codegen::Report,
    prelude::{Fancy, FancySettings, Fancyness, Present, RenderMut, Report, Textual},
};
use aldebaran_span::prelude::Span;

#[derive(Debug, Report)]
#[error("unexpected token")]
#[report(title = "unexpected token", message = "expected an expression")]
struct Unexpected(Span);

fn main() {
    let source = "let answer = ;";
    let error = Unexpected(Span::unit(13));
    let attached = Report::attach(&error, source);

    let mut output = String::new();
    let mut renderer = Textual::<'_, _, Fancy, _>::new(&mut output);
    let settings = Present::from_input(FancySettings::level(Fancyness::Outblown));

    renderer
        .render_mut_with_input(settings, &attached)
        .expect("diagnostic rendering succeeds");

    println!("{output}");
}
