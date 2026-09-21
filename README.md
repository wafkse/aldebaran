# Aldebaran Compiler Framework

> [!WARNING]
> **Aldebaran is a work in progress.** It is currently in a crude parsing-only stage. Further compiler work is pending.

**Aldebaran** is a compiler framework written in Rust for source processing, diagnostics, parsing, and supporting compiler infrastructure.

The runtime framework is `no_std` compatible.

## Status

Current work is focused on parsing and its supporting infrastructure. Compiler stages beyond parsing are not implemented yet.

## Parsing

Parsing is currently direct and source-oriented. `Text` tracks source position while assertions describe what may be consumed.

```rust
use aldebaran_text::prelude::{AsciiAlphabetic, AsciiAlphanumeric, Text};

let mut text = Text::create("aldebaran42");

let identifier_start = text
    .expect_is_next(AsciiAlphabetic)
    .expect("identifier start");

text.ignore_while(AsciiAlphanumeric);

assert_eq!(identifier_start, 'a');
assert_eq!(text.peek(), None);
```

## Errors

Failed assertions produce structured errors with their source span intact.

```rust
use aldebaran_report::prelude::{Annotated, Report};
use aldebaran_text::prelude::{AsciiDigit, Text};

let mut text = Text::create("x");

let error = text
    .expect_is_next(AsciiDigit)
    .expect_err("a digit was expected");

assert_eq!(Report::title(&error), "unexpected character");
assert_eq!(Annotated::target(&error).start(), 0);
```

## Diagnostics

Custom diagnostics can derive their report structure. The derive is available through the `report-codegen` feature.

```rust
use aldebaran::report::codegen::Report;
use aldebaran::span::prelude::Span;

#[derive(Debug, Report)]
#[error("unexpected token")]
#[report(title = "unexpected token", message = "expected an expression")]
struct Unexpected(Span);

let error = Unexpected(Span::MIN);
```

Fancy diagnostics render source context, spans, and annotations. ANSI coloring is omitted below.

```text
error: you hate snickers?? so do I!!
    ┌─[<unknown>:1:2]
 1  │ snicker
    │ ^^^^^^^
 2  │  dog bone
    │ ^^^^^^^^^
    │   [^]: you hate snickers?? so do I!!
    │
────┘
```

## Custom Assertions

Parsing predicates are ordinary types implementing `Assert`.

```rust
use core::fmt;

use aldebaran_logic::{
    fmt::{Formatter, Precedence},
    prelude::Assert,
};
use aldebaran_text::prelude::Text;

#[derive(Debug, Clone, Copy)]
struct Lowercase;

impl Assert<char> for Lowercase {
    fn assert(&self, input: char) -> bool {
        input.is_ascii_lowercase()
    }

    fn output_with<W, F>(_: &Self, writer: &mut W, _: Precedence) -> fmt::Result
    where
        W: fmt::Write,
        F: Formatter,
    {
        writer.write_str("<lowercase>")
    }
}

let mut text = Text::create("a");
text.expect_is_next(Lowercase).expect("lowercase character");
```

## History

Aldebaran was originally part of another project and was later extracted into this standalone repository. The initial commit series reflects that extraction and may appear unusually segmented.

# License

Copyright (C) 2026 W. Frakchi

This program is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

See [the full license agreement](./LICENSE) for further information.
