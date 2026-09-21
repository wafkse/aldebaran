# Aldebaran Visualize

`aldebaran-visualize` provides lossy human-readable rendering for source-shaped values.

Visualization is intentionally distinct from ordinary printing. A byte can print as its numeric value while visualizing as source text. ASCII bytes retain their character representation and non-ASCII bytes use the Unicode replacement character.

The `Visualize` trait writes directly into any `core::fmt::Write` sink without allocation. Its `Visual` view exposes the same visualization through both `core::fmt::Display` and `core::fmt::Debug` when standard formatting integration is needed.

The crate has no dependencies and supports `no_std` consumers.
