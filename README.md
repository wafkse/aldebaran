# Aldebaran Compiler Framework

![Development status](https://img.shields.io/badge/status-very%20pre--alpha-red)

Aldebaran is a set of Rust crates for source processing and compiler frontends.

The workspace currently covers source and span handling, lexing, token streams,
internment, typed IDs, diagnostics, and terminal/text rendering. Most of it can
be used either through the top-level `aldebaran` crate or as individual
`aldebaran-*` crates.

> [!WARNING]
> Aldebaran is **very pre-alpha**. Expect breaking changes, incomplete features,
> and bugs. Releases are intended for experimentation.
>
> THIS IS BEING REWRITTEN

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

The example crates are not present in this working tree. The runtime crates
remain independently buildable, and the code below shows their current public
module paths.

## Crates

Use `aldebaran` if you want the whole framework. The individual crates are
also public and can be depended on directly.

- **Facade:** [`aldebaran`](crates/aldebaran)
- **Foundations and identity:** [`aldebaran-baseline`](crates/aldebaran-baseline),
  [`aldebaran-ice`](crates/aldebaran-ice), [`aldebaran-id`](crates/aldebaran-id)
- **Source processing:** [`aldebaran-source`](crates/aldebaran-source),
  [`aldebaran-interner`](crates/aldebaran-interner), [`aldebaran-grammar`](crates/aldebaran-grammar),
  [`aldebaran-text`](crates/aldebaran-text)
- **Logic and presentation:** [`aldebaran-logic`](crates/aldebaran-logic),
  [`aldebaran-print`](crates/aldebaran-print), [`aldebaran-ansi`](crates/aldebaran-ansi)
- **Diagnostics:** [`aldebaran-report`](crates/aldebaran-report)
- **Derive implementations:** [`aldebaran-logic-macro`](crates/aldebaran-logic/aldebaran-logic-macro),
  [`aldebaran-report-macro`](crates/aldebaran-report/aldebaran-report-macro)

Allocator support is an optional feature of `aldebaran-baseline`. The proc-macro
packages are implementation dependencies of their corresponding runtime crates.

## Source processing

The lower-level text API keeps track of source position while assertions decide
what can be consumed:

```rust,ignore
use aldebaran::text::prelude::{AsciiAlphabetic, AsciiAlphanumeric, Cursor};

let mut cursor = Cursor::create("aldebaran42");

let first = cursor
    .expect_is_next(AsciiAlphabetic)
    .expect("identifier start");

cursor.ignore_while(AsciiAlphanumeric);

assert_eq!(first, 'a');
assert_eq!(cursor.peek(), None);
```

The two-phase lexer contract is available at
[`aldebaran::text::cursor::consume::Lex`](crates/aldebaran-text/src/cursor/consume.rs).

## Diagnostics

Reports carry their source locations separately from rendering. The
`report-codegen` feature adds the derive macro:

```rust,ignore
use aldebaran::{report::codegen::Report, baseline::span::Span};

#[derive(Debug, Report)]
#[error("unexpected token")]
#[report(title = "unexpected token", message = "expected an expression")]
struct Unexpected(Span);
```

The renderers can add source context, annotations, multiple spans, and terminal
styling. `report-codegen` enables the derive macro in the facade; the report
runtime remains usable without it.

This rendered diagnostic illustrates UTF-8 source locations and overlapping
annotations with Unicode framing and ANSI color:

![ANSI-colored duplicate-binding diagnostic with overlapping UTF-8 spans](https://raw.githubusercontent.com/wafkse/aldebaran/development/docs/diagnostics-ansi.svg)

<details>
<summary>Plain-text rendering (<code>--no-ansi</code>)</summary>

With ANSI styling stripped, the same diagnostic is:

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

</details>

## Examples

Runnable example packages are not included in this working tree. The runtime
tests cover the crate-level behavior; the examples can be restored as separate
workspace members when their migration is ready.

## Compatibility

The `aldebaran` facade is `#![no_std]`. Crates that need owned storage use
`alloc`. Examples and tests use `std`.

The repository uses stable Rust by default. CI also builds and tests on beta and
nightly, on Linux, macOS, and Windows. There is no separately declared MSRV at
the moment.

## Project status

The source-processing, parsing, diagnostic, and presentation pieces can be
tried today. Aldebaran is still very pre-alpha, with no stable API or migration
guarantee between experimental releases.

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

See [the full license agreement](https://github.com/wafkse/aldebaran/blob/development/LICENSE) for further information.
