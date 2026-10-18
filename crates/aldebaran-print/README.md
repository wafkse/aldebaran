# Aldebaran Printing

`aldebaran-print` formats values into caller-provided sinks using external
context. For example, an interned identifier can resolve its text through an
interner at rendering time, so the value need not retain that storage or
implement [`Display`]. `Print` supplies this contract and adapters
for standard formatting.

Output composition preserves each subject's context requirements until
rendering. Aggregation shares a context, sequencing pairs independent contexts,
and repetition reuses a context without adding separators. Since writes go
directly to the sink, a formatting failure can leave partially written output.

`Visualize` supplies source views without contextual lookup. Its byte view
preserves byte boundaries and replaces each non-ASCII byte independently, while
string views preserve their text. This makes it suitable for displaying source
content independently of stored-name resolution.

`Stylus` adds presentation around a writer closure.
[`aldebaran-ansi`](https://docs.rs/aldebaran-ansi) uses it to provide terminal
styling alongside contextual printing.

The crate is `no_std` and writes into caller-provided formatting sinks.

[`Display`]: https://doc.rust-lang.org/core/fmt/trait.Display.html
