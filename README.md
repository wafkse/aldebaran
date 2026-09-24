# Aldebaran Compiler Framework

Aldebaran is a set of Rust crates for source processing and compiler frontends.

The workspace currently covers source and span handling, lexing, token streams,
internment, typed IDs, diagnostics, and terminal/text rendering. Most of it can
be used either through the top-level `aldebaran` crate or as individual
`aldebaran-*` crates.

> [!WARNING]
> Aldebaran is pre-1.0 and under active development. Public APIs are not stable
> yet.

## What exists today

| Area | Status |
| --- | --- |
| Sources and spans | Implemented |
| Lexing and text processing | Implemented |
| Token streams and parser lookahead | Implemented |
| Assertions and token classification | Implemented |
| Internment and typed IDs | Implemented |
| Structured diagnostics | Implemented |
| Diagnostic derives | Optional, via `report-codegen` |
| Terminal/text presentation | Implemented |
| Language semantics | Left to the consumer |
| Shared IR/backend | Not present |

Aldebaran does not try to define the semantic phases of every language built on
top of it. Type checking, lowering, evaluation, or other language-specific work
can live in the consuming project.

## mini-lisp

[`mini-lisp`](./examples/mini-lisp) is the best place to see the
pieces working together. It has a lexer, persistent symbol internment, a live
token stream, a recursive-descent parser, a spanned AST, an evaluator, and
diagnostics for lexing, parsing, and evaluation errors.

```sh
cargo run -p aldebaran-example-mini-lisp
```

The example also uses derived reports and multi-span annotations.

## Crates

Use `aldebaran` if you want the whole framework. The individual crates are
also public and can be depended on directly.

- **Facade:** [`aldebaran`](./aldebaran)
- **Source/core:** [`aldebaran-span`](./aldebaran-span),
  [`aldebaran-source`](./aldebaran-source),
  [`aldebaran-id`](./aldebaran-id),
  [`aldebaran-interner`](./aldebaran-interner),
  [`aldebaran-primitive`](./aldebaran-primitive),
  [`aldebaran-dsa`](./aldebaran-dsa)
- **Language processing:** [`aldebaran-logic`](./aldebaran-logic),
  [`aldebaran-text`](./aldebaran-text),
  [`aldebaran-grammar`](./aldebaran-grammar)
- **Diagnostics:** [`aldebaran-report`](./aldebaran-report)
- **Presentation:** [`aldebaran-print`](./aldebaran-print),
  [`aldebaran-style`](./aldebaran-style),
  [`aldebaran-ansi`](./aldebaran-ansi),
  [`aldebaran-visualize`](./aldebaran-visualize)
- **Support:** [`aldebaran-heap`](./aldebaran-heap),
  [`aldebaran-hash`](./aldebaran-hash),
  [`aldebaran-ice`](./aldebaran-ice)

The proc-macro/codegen crates are implementation details for the public crates
above.

## Source processing

The lower-level text API keeps track of source position while assertions decide
what can be consumed:

```rust
use aldebaran::text::prelude::{AsciiAlphabetic, AsciiAlphanumeric, Text};

let mut text = Text::create("aldebaran42");

let first = text
    .expect_is_next(AsciiAlphabetic)
    .expect("identifier start");

text.ignore_while(AsciiAlphanumeric);

assert_eq!(first, 'a');
assert_eq!(text.peek(), None);
```

For the full lexing path, including internment and `LexStream`, see
[`lexing`](./examples/lexing).

## Diagnostics

Reports carry their source locations separately from rendering. The
`report-codegen` feature adds the derive macro:

```rust
use aldebaran::{report::codegen::Report, span::prelude::Span};

#[derive(Debug, Report)]
#[error("unexpected token")]
#[report(title = "unexpected token", message = "expected an expression")]
struct Unexpected(Span);
```

The renderers can add source context, annotations, multiple spans, and terminal
styling. See [`diagnostics`](./examples/diagnostics) and
[`mini-lisp`](./examples/mini-lisp) for complete examples.

The diagnostics example uses UTF-8 identifiers and deliberately overlapping
annotations. It renders this without ANSI color:

```text
error: duplicate binding
    ┌─[<unknown>:1:3]
 1  │ let résumé = "draft";
    │     ^^^^^^
    │   [^]: the first declaration is here
    │
 2  │ let résumé = résumé + " final";
    │     ******   ######
    │              ~~~~~~~~~~~~~~~~~
    │ @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@
    │   [*]: `résumé` is declared again here
    │
    │   [#]: this read refers to the binding being shadowed
    │
    │   [~]: this initializer belongs to the redeclaration
    │
    │   [@]: the redeclaration spans this statement
    │
 3  │ emit(résumé);
    │      ------
    │   [-]: this later use would resolve through the redeclaration
    │
────┘
```

## More examples

The [`examples`](./examples) directory also contains:

| Example | Covers |
| --- | --- |
| [`lexing`](./examples/lexing) | Source traversal, assertions, lexing, internment, token streams, lookahead |
| [`diagnostics`](./examples/diagnostics) | UTF-8 source, overlapping annotations, multi-span reports, diagnostic rendering |
| [`presentation`](./examples/presentation) | Printing, styles, ANSI output, visualization |
| [`foundations`](./examples/foundations) | IDs, spans, primitives, collections, hashing, heap support |

See [`examples/README.md`](./examples/README.md) for the full coverage map.

## Compatibility

The `aldebaran` facade is `#![no_std]`. Crates that need owned storage use
`alloc`. Examples and tests use `std`.

The repository uses stable Rust by default. CI also builds and tests on beta and
nightly, on Linux, macOS, and Windows. There is no separately declared MSRV at
the moment.

## Project status

The source-processing, parsing, diagnostic, and presentation pieces are usable
today, but the APIs are still changing.

There is no shared IR, optimizer, or backend layer in the workspace currently.
Those are not required for using Aldebaran as frontend infrastructure.

## History

Aldebaran was extracted from an existing project into this repository. The
initial commit series is therefore an import and cleanup of existing work, not
the chronological development history of the framework.

## License

Copyright (C) 2026 W. Frakchi

This program is free software: you can redistribute it and/or modify it under
the terms of the GNU Affero General Public License as published by the Free
Software Foundation, either version 3 of the License, or (at your option) any
later version.

See [the full license agreement](./LICENSE) for further information.
