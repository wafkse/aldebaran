# Aldebaran

The `aldebaran` crate re-exports the framework crates under namespaces such as
`aldebaran::text` and `aldebaran::report`. Depend on individual crates when an
application needs only part of the framework.

The facade is `no_std` and has no default features. Enable optional capabilities
as needed.

- `report-codegen` enables the report derive.
- `source-storage` enables source-aware interning.

## Source coordinates

For string sources, text cursors consume Unicode scalar values while spans
count UTF-8 bytes. Byte sources use bytes for both components and coordinates.
Keeping source units consistent lets lexical values and diagnostics refer to
the same source locations without treating character counts as byte offsets.

```rust
use aldebaran::text::prelude::{AsciiAlphabetic, AsciiAlphanumeric, Cursor};

let mut cursor = Cursor::create("aldebaran42");

let first = cursor
    .expect_is_next(AsciiAlphabetic)
    .expect("identifier start");

cursor.ignore_while(AsciiAlphanumeric);

assert_eq!(first, 'a');
assert_eq!(cursor.peek(), None);
```

Lexical types implement
[`Lex`](https://github.com/wafkse/aldebaran/tree/development/crates/aldebaran-text/src/cursor/consume.rs),
which separates recognition from construction with caller-owned context. Each
implementation defines cursor advancement on recognition failure, and the
default combined operation does not roll back cursor or context changes.

## Diagnostics

Reports can retain annotation spans without borrowing source text. Attach the
source that those spans describe when rendering. Enable `report-codegen` to
derive the report traits for error types such as the following.

```rust
# #[cfg(feature = "report-codegen")]
# fn main() {
use aldebaran::{report::codegen::Report, baseline::span::Span};

#[derive(Debug, Report)]
#[error("unexpected token")]
#[report(title = "unexpected token", message = "expected an expression")]
struct Unexpected(Span);
# }
# #[cfg(not(feature = "report-codegen"))]
# fn main() {}
```

![ANSI-colored duplicate-binding diagnostic with overlapping UTF-8 spans](https://raw.githubusercontent.com/wafkse/aldebaran/development/docs/diagnostics-ansi.svg)

See the [workspace README](https://github.com/wafkse/aldebaran#readme) for the
project overview and crate map.
