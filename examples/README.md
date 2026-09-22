# Aldebaran examples

This package contains runnable examples for the public workspace surface. The
examples are grouped around real compiler-framework workflows so related crates
are shown together.

Run an example with

```sh
cargo run -p aldebaran-examples --bin foundations
```

The available examples are

| Example | Workspace coverage |
| --- | --- |
| `foundations` | `aldebaran-primitive`, `aldebaran-dsa`, `aldebaran-hash`, `aldebaran-heap`, `aldebaran-ice`, `aldebaran-id`, `aldebaran-span` |
| `source_and_text` | Complete `Lex` pipeline using source slices, persistent identifier internment, integer construction, `LexStream`, derived lexical assertions, trivia filtering, and visualization. |
| `logic_and_grammar` | Advanced `Assert` and `Choose` derives using conjunctive, disjunctive, expression-backed, and enum-variant predicates, then `OneOf` classification and grammar-stream lookahead. |
| `mini_lisp` | End-to-end Lisp built around `Lex<str>` and a live `LexStream`, with composable `Assert` predicates, persistent symbol internment, direct stream parsing into a spanned AST, evaluation, structured phase errors, derived `Report` implementations, multi-span annotations, and fancy source diagnostics. |
| `presentation` | `aldebaran-print`, `aldebaran-style`, `aldebaran-ansi`, `aldebaran-visualize` |
| `diagnostics` | `aldebaran-report`, `aldebaran-report-macro`, `aldebaran-report-codegen`, `aldebaran-source`, `aldebaran-span` |
| `facade` | `aldebaran` and its source-processing facade |

`aldebaran-primitive-core` and `aldebaran-primitive-macro` provide the public
primitive API through `aldebaran-primitive`. The `foundations` example exercises
that combined surface.
