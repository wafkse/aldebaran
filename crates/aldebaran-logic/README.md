# Aldebaran Logic

`aldebaran-logic` provides predicate assertions and typed classification for
Aldebaran frontends. Its combinators preserve each branch's classification
result while composing recognition rules, so callers can retain the distinction
between checking an input and identifying what it matched.

Classifiers must agree with their predicates about acceptance. Conjunction
retains both classifications, disjunction retains branch identity with priority
for the left operand, and exclusive disjunction rejects overlapping matches.
Assertion descriptions carry operator precedence so formatters can preserve
grouping in expectation messages.

[`aldebaran-text`](https://docs.rs/aldebaran-text/latest/aldebaran_text/)
uses these contracts for byte and character recognition. To make the same
contracts available alongside the other frontend components, the runtime facade
[`aldebaran`](https://docs.rs/aldebaran/latest/aldebaran/) re-exports the logic
traits, combinators, and derives.
