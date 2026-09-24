# Aldebaran examples

Each example is a separate workspace crate with only the dependencies it uses.
Run them from the workspace root with `cargo run -p <package>`.

| Crate | Package | What it covers |
| --- | --- | --- |
| [`mini-lisp`](./mini-lisp) | `aldebaran-example-mini-lisp` | End-to-end lexer, internment, token streaming, recursive-descent parser, spanned AST, evaluator, and phase-specific diagnostics. Uses the `aldebaran` facade. |
| [`lexing`](./lexing) | `aldebaran-example-lexing` | Source traversal, derived lexical assertions, `Lex`, persistent internment, trivia filtering, and parser lookahead over the resulting token stream. |
| [`diagnostics`](./diagnostics) | `aldebaran-example-diagnostics` | A derived report over UTF-8 source with six labels and nested overlapping spans. CLI flags independently toggle ANSI styling and Unicode framing. |
| [`presentation`](./presentation) | `aldebaran-example-presentation` | Structured printing, lossy visualization, ANSI painting, and direct style application. |
| [`foundations`](./foundations) | `aldebaran-example-foundations` | Primitive casts, spans, inline collections, reverse maps, hashing, heap allocation, typed IDs, and ICE helpers. |

For example:

```sh
cargo run -p aldebaran-example-mini-lisp
cargo run -p aldebaran-example-lexing
cargo run -p aldebaran-example-diagnostics
```

The diagnostics example defaults to ANSI color with Unicode framing. Use
`--ansi` / `--no-ansi` and `--unicode` / `--no-unicode` to control those two
rendering choices independently; `--help` prints the complete CLI usage.

The examples overlap at their boundaries. `mini-lisp` shows how the framework
composes through the facade; the smaller crates focus on individual subsystems
and use those crates directly.
