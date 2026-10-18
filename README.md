# Aldebaran

Aldebaran is a Rust framework for compiler frontends that keep lexical values,
source coordinates, and diagnostics connected. It provides source traversal,
lexical recognition, token-stream composition, interning, and diagnostic rendering
as separate crates, so applications can adopt the capabilities they need.

Source traits define the coordinate system shared by frontend operations and
reports. Text and grammar crates provide recognition mechanisms, while reports
carry annotations for renderers to interpret against a source. Printing resolves
context-dependent values at rendering time, and ANSI styling adds terminal
presentation through the same output interface.

**Status** Pre-alpha. APIs may change and some features are incomplete.

## Crates

| Crate | Role |
| --- | --- |
| [`aldebaran`](crates/aldebaran) | Framework facade |
| [`aldebaran-source`](crates/aldebaran-source) | Source locations and line views |
| [`aldebaran-text`](crates/aldebaran-text) | Text cursors and lexical recognition |
| [`aldebaran-grammar`](crates/aldebaran-grammar) | Grammar and token-stream composition |
| [`aldebaran-interner`](crates/aldebaran-interner) | Source-aware interning |
| [`aldebaran-report`](crates/aldebaran-report) | Diagnostic reports and renderers |
| [`aldebaran-print`](crates/aldebaran-print) | Contextual printing and output composition |
| [`aldebaran-ansi`](crates/aldebaran-ansi) | ANSI terminal styling |
| [`aldebaran-baseline`](crates/aldebaran-baseline) | Shared types and collections |
| [`aldebaran-id`](crates/aldebaran-id) | Typed identifiers and collections |
| [`aldebaran-ice`](crates/aldebaran-ice) | Internal compiler error support |
| [`aldebaran-logic`](crates/aldebaran-logic) | Logical assertion and selection traits |
| [`aldebaran-report-macro`](crates/aldebaran-report/aldebaran-report-macro) | Report derive |
| [`aldebaran-logic-macro`](crates/aldebaran-logic/aldebaran-logic-macro) | Logic derives |

See the [`aldebaran` README](crates/aldebaran/README.md) for facade features and usage.

Copyright (C) 2026 W. Frakchi. Licensed under AGPL-3.0-or-later. See [LICENSE](LICENSE).
