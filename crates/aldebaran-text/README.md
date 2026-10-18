# Aldebaran Text Processing

`aldebaran-text` helps compiler frontends recognize lexical forms from byte and
character sources while preserving source positions for diagnostics. Because
its lexer interface separates recognition from semantic construction, parsers
can delay conversion until the surrounding syntax is accepted.

For built-in `str` and byte-slice sources, cursor offsets and spans count bytes,
while lookahead counts source components. Required reads consume mismatches,
validation preserves the position, and optional reads consume only matches.
Branches keep cursor progress speculative until explicitly committed, while
lexical construction has no automatic rollback of cursor or context changes.

The resulting lexical values can be consumed through `aldebaran-grammar`,
allowing a parser to use the same token-stream interface with lexical output
or with producers that do not depend on this crate.
