# Aldebaran Printing

`aldebaran-print` formats values into caller-provided sinks using external
context. For example, an interned identifier can resolve its text through an
interner at rendering time, so the value need not retain that storage or
implement [`core::fmt::Display`]. `Print` supplies this contract and adapters
for standard formatting.

Output composition preserves each subject's context requirements until
rendering. Aggregation shares a context, sequencing pairs independent contexts,
and repetition reuses a context without adding separators. Since writes go
directly to the sink, a formatting failure can leave partially written output.

`Visualize` supplies source views without contextual lookup. Its byte view
preserves byte boundaries and replaces each non-ASCII byte independently, while
string views preserve their text. `Stylus` surrounds a writer closure with
presentation, and [`aldebaran-ansi`](https://docs.rs/aldebaran-ansi) implements
terminal styling through that contract. The printing crate remains `no_std`.
