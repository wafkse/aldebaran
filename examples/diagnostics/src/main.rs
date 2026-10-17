//! Multi-span diagnostic report construction and rendering.

use core::num::NonZero;
use std::{env, process::ExitCode};

use aldebaran_report::{
    codegen::Report,
    prelude::{Fancy, FancySettings, InlineAnnotations, Label, Present, RenderMut, Report, Textual},
    render::textual::offload::fancy::{Colored, LayoutSettings, charset::Charset},
};
use aldebaran_span::prelude::Span;

/// Command-line options accepted by the diagnostic demonstration.
const USAGE: &str = "\
Usage: aldebaran-example-diagnostics [OPTIONS]\n\
\n\
Render a Unicode duplicate-binding diagnostic with overlapping labels.\n\
\n\
Options:\n\
    --ansi          enable ANSI color/style escapes (default)\n\
    --no-ansi       disable ANSI color/style escapes\n\
    --unicode       use Unicode diagnostic framing (default)\n\
    --no-unicode    use ASCII-only diagnostic framing\n\
    -h, --help      print this help\n";

/// Independent color and glyph policies selected by the command line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Config {
    /// Whether output includes ANSI styling escapes.
    ansi: bool,

    /// Whether diagnostic framing uses Unicode glyphs.
    unicode: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self { ansi: true, unicode: true }
    }
}

/// The requested action after parsing command-line options.
#[derive(Debug, PartialEq, Eq)]
enum Cli {
    /// Render the demonstration with the selected presentation settings.
    Run(Config),
    /// Display usage without rendering a diagnostic.
    Help,
}

impl Config {
    /// Parse presentation switches, rejecting unknown options.
    fn parse(args: impl IntoIterator<Item = String>) -> Result<Cli, String> {
        let mut config = Self::default();

        for argument in args {
            match argument.as_str() {
                "--ansi" => config.ansi = true,
                "--no-ansi" => config.ansi = false,
                "--unicode" => config.unicode = true,
                "--no-unicode" => config.unicode = false,
                "-h" | "--help" => return Ok(Cli::Help),
                _ => return Err(format!("unknown option `{argument}`")),
            }
        }

        Ok(Cli::Run(config))
    }

    /// Project the ANSI switch into the renderer's color policy.
    const fn colored(self) -> Colored {
        if self.ansi { Colored::Yes } else { Colored::No }
    }

    /// Select framing glyphs independently of the ANSI color policy.
    const fn charset(self) -> Charset {
        if self.unicode { Charset::UNICODE } else { Charset::ASCII }
    }
}

/// A duplicate-binding report with primary and related source annotations.
#[derive(Debug, Report)]
#[error("duplicate binding")]
#[report(title = "duplicate binding", annotations = annotations)]
struct DuplicateBinding {
    /// The redeclaration target and five related source labels.
    annotations: InlineAnnotations<Label<&'static str>, 5>,
}

impl DuplicateBinding {
    /// Label the redeclaration and the source locations explaining its effects.
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

/// Locate a nonempty fragment known to occur in the example source.
fn span_of(source: &str, needle: &str) -> Span {
    let start = source.find(needle).expect("example source contains the requested fragment");
    let length = NonZero::new(needle.len()).expect("example fragment is nonempty");

    Span::new(start, length)
}

/// Locate a nonempty fragment after a known UTF-8 source boundary.
fn span_of_after(source: &str, needle: &str, after: usize) -> Span {
    let tail = source.get(after..).expect("search starts at a UTF-8 boundary");
    let relative = tail.find(needle).expect("example source contains the requested fragment");
    let length = NonZero::new(needle.len()).expect("example fragment is nonempty");

    Span::new(after + relative, length)
}

/// Render the fixed multi-span diagnostic using the selected presentation policy.
fn render(config: Config) -> String {
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
    let settings = FancySettings::tuple((config.colored(), LayoutSettings::standard(), config.charset()));

    renderer
        .render_mut_with_input(Present::from_input(settings), &attached)
        .expect("diagnostic rendering succeeds");

    output
}

fn main() -> ExitCode {
    let config = match Config::parse(env::args().skip(1)) {
        Ok(Cli::Run(config)) => config,
        Ok(Cli::Help) => {
            print!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        Err(error) => {
            eprintln!("error: {error}\n\n{USAGE}");
            return ExitCode::from(2);
        }
    };

    println!("{}", render(config));

    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::{Cli, Config, render};

    #[test]
    fn defaults_to_unicode_with_ansi() {
        let parsed = Config::parse([]).expect("default CLI parses");
        assert_eq!(parsed, Cli::Run(Config::default()));
        let Cli::Run(config) = parsed else { return };

        assert_eq!(config, Config::default());

        let output = render(config);

        assert!(output.contains('┌'));
        assert!(output.contains('\u{1b}'));
    }

    #[test]
    fn ansi_and_unicode_can_be_toggled_independently() {
        let parsed = Config::parse(["--ansi".into(), "--no-unicode".into()]).expect("CLI parses");
        assert_eq!(
            parsed,
            Cli::Run(Config {
                ansi: true,
                unicode: false
            })
        );
        let Cli::Run(config) = parsed else { return };

        let output = render(config);

        assert!(output.contains('\u{1b}'));
        assert!(output.contains("+-[<unknown>:1"));
        assert!(!output.contains('┌'));
    }

    #[test]
    fn later_toggle_overrides_earlier_toggle() {
        let parsed = Config::parse(["--no-ansi".into(), "--ansi".into(), "--no-unicode".into(), "--unicode".into()]).expect("CLI parses");
        assert_eq!(parsed, Cli::Run(Config::default()));
        let Cli::Run(config) = parsed else { return };

        assert_eq!(config, Config::default());
    }
}
